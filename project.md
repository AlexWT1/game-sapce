# Browser Time Tracking — feasibility

## Вопрос

Можно ли сделать в Game Space отслеживание времени в браузере по сайтам?

Пример сценария:

1. Пользователь запускает Google Chrome через лаунчер.
2. Смотрит фильм на YouTube 3 часа.
3. В лаунчере за сегодня: **Google Chrome — 3 ч**.
4. Самый активный сайт за сессию: **youtube.com**.

## Короткий ответ

**Да, можно.** Общее время браузера — уже почти готово (есть process monitoring через `sysinfo`). Время по сайтам/вкладкам требует отдельного механизма получения активного URL, но это реализуемо локально, без облака.

---

## Уровень 1 — общее время браузера (просто)

Реализуется на существующей архитектуре:

1. Добавить браузер как приложение в библиотеку (или отдельный тип «Browser»).
2. При запуске через лаунчер создать сессию (`browser_sessions` или переиспользовать `game_sessions` с `source = 'Browser'`).
3. Мониторить PID процесса (как игр) каждые 1–3 секунды.
4. При завершении сохранить `duration_seconds`.

Ограничение: у браузеров часто multi-process (renderer, GPU). Нужно мониторить **главный PID** (и, возможно, группу процессов), а не один renderer.

Результат UI:

```text
Сегодня
Google Chrome   3 ч 02 мин
```

---

## Уровень 2 — время по сайтам (нужен источник URL)

Проблема: OS не даёт «активный URL вкладки» просто так. Нужен один из подходов.

### Вариант A — browser extension + Native Messaging (рекомендуется)

Расширение (Chrome/Firefox MV3) подписывается на `tabs.onActivated` / `tabs.onUpdated` / `visibility` и шлёт события:

```json
{ "url": "https://youtube.com/watch=...", "tabId": 1, "event": "activated", "ts": 1730000000 }
```

Путь данных:

```text
Extension
  → chrome.runtime.sendNativeMessage("com.gamespace.tracker")
    → Native Messaging Host ( exe / rust binary )
      → Tauri app ( localhost IPC или named pipe / socket )
        → SQLite
```

Native Messaging Host регистрируется в:

- Windows: `HKCU\Software\Google\Chrome\NativeMessagingHosts\...`
- Linux: `~/.config/google-chrome/NativeMessagingHosts/...`
- macOS: `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/...`

**Плюсы:** точный URL, активная вкладка, можно фильтровать «только foreground».
**Минусы:** нужно ставить расширение, поддержка каждого браузера отдельно.

### Вариант B — локальный HTTP-сервер в Game Space

Расширение шлёт `POST http://127.0.0.1:<port>/event` (нужен фиксированный порт + токен в extension settings, чтобы чужие сайты не спамили).

**Плюсы:** проще, чем native messaging.
**Минусы:** порт может быть занят; нужен строгий origin/token check.

### Вариант C — разбор заголовка окна (без расширения)

Через `sysinfo` / platform APIs читать title окна:

```text
"video name - YouTube - Google Chrome"
```

**Плюсы:** без расширения, кросс-браузерно грубо.
**Минусы:** ненадёжно (некоторые браузеры не отдают URL в title), нет точного domain, poor UX для «youtube.com».

### Вариант D — чтение browser history

SQLite history браузера (`History` Chrome).

**Плюсы:** просто.
**Минусы:** history ≠ активное время; нельзя узнать, сколько реально сидели на вкладке. Непригодно для «3 ч на YouTube».

### Вариант E — Chrome DevTools Protocol / remote debugging

Запуск браузера с `--remote-debugging-port`.

**Плюсы:** полный доступ к tabs.
**Минусы:** небезопасно, не для обычного пользователя, Firefox/Edge отличаются. Не рекомендуется как основной путь.

---

## Рекомендуемая архитектура для Game Space

```text
┌─────────────────┐     native msg / HTTP      ┌──────────────────┐
│ Browser         │ ─────────────────────────► │ BrowserTracker   │
│ extension       │                            │ (Rust service)   │
└─────────────────┘                            └────────┬─────────┘
                                                       │ events
                                              ┌────────▼─────────┐
                                              │ SQLite           │
                                              │ browser_sessions │
                                              │ site_visits      │
                                              └────────┬─────────┘
                                                       │
                                              ┌────────▼─────────┐
                                              │ React UI         │
                      Tauri commands/events   │ Statistics / Home│
┌─────────────────┐ ────────────────────────► └──────────────────┘
│ Process monitor │  (browser open/close, total time)
│ (sysinfo)       │
└─────────────────┘
```

### Модель данных (через migrations)

```sql
-- браузер как «приложение»
-- games.source = 'Browser', executable_path = путь к chrome.exe

CREATE TABLE browser_sessions (
    id INTEGER PRIMARY KEY,
    game_id INTEGER NOT NULL,          -- ссылка на браузер в games
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_seconds INTEGER NOT NULL DEFAULT 0,
    pid INTEGER
);

CREATE TABLE site_visits (
    id INTEGER PRIMARY KEY,
    session_id INTEGER NOT NULL REFERENCES browser_sessions(id),
    domain TEXT NOT NULL,              -- youtube.com
    url TEXT,                          -- опционально, можно обрезать query
    started_at TEXT NOT NULL,
    ended_at TEXT,
    duration_seconds INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_site_visits_day ON site_visits(started_at, domain);
```

Время по сайтам = сумма `duration_seconds` по `domain` за день (foreground only).

### Агрегация для UI

```text
Сегодня: Google Chrome — 3 ч 02 мин

Сайты:
  youtube.com   2 ч 48 мин
  github.com     11 мин
  ...
```

---

## Как считать время на сайте

На каждый event от extension:

1. `activated(url)` → закрыть предыдущий open visit (`ended_at = now`, `duration = now - started_at`), открыть новый.
2. `deactivated` / window blur / tab hidden → закрыть текущий visit.
3. Timer тикает только когда **foreground активной вкладки**.

Не считать background-вкладки — иначе «открыт 40 вкладок» даст ложные часы.

### Пагинация/шум

- Нормализовать URL → domain (`www.` убрать, path опционально).
- YouTube: считать `youtube.com`, не каждый query param.
- Можно whitelist/blacklist доменов в настройках.

---

## Интеграция с запуском через лаунчер

1. Пользователь жмёт «Играть» у Google Chrome → `launch_browser`.
2. Старт `browser_sessions`, PID = main browser process.
3. Extension подключается к трекеру (или HTTP).
4. События site_visits привязываются к текущей `session_id`.
5. Браузер закрыт → закрыть session + все открытые visits.

Если браузер открыли **не через лаунчер**:

- Вариант 1: считать только запуски через лаунчер (проще, соответствует примеру).
- Вариант 2: background watcher по известным browser процессам (сложнее, опционально later).

---

## Приватность (важно)

- Все данные **локально** в SQLite, без отправки в сеть.
- Настройка: «Отслеживать сайты» on/off.
- Опция хранить только `domain`, не полный URL (без query/поисковых строк).
- Опция исключить инкогнито / приватные окна (extension не должна слать события от incognito, если не разрешено).
- Экспорт/удаление — через существующий Data settings.

---

## Сложность и этапы

| Этап | Что | Сложность |
|------|-----|-----------|
| 1 | Browser как app + total time через process monitor | Низкая |
| 2 | UI: «Браузеры сегодня» в Statistics/Home | Низкая |
| 3 | Local tracker (HTTP или socket) в Rust | Средняя |
| 4 | Chrome extension (foreground tab events) | Средняя |
| 5 | site_visits + агрегация по domain | Средняя |
| 6 | Native messaging host + установка расширения из лаунчера | Выше |
| 7 | Firefox / Edge / Safari | Платформенно |

**MVP:** этапы 1–5 (extension можно временно слать на local HTTP с токеном).
**Production:** native messaging, автоустановка extension, multi-browser.

---

## Платформенные нюансы

- **Windows:** Chrome/Edge — native messaging ок; title-based fallback проще.
- **Linux:** Chrome/Chromium/Firefox — пути к manifest отличаются.
- **macOS:** Safari extension API сложнее; для Safari MVP ограничиться временем процесса или не поддерживать.

---

## Вывод

| Что | Возможность |
|-----|-------------|
| Время браузера в лаунчере | Да, дёшево, на текущем `sysinfo` monitor |
| Время по сайтам (youtube.com) | Да, через extension + local tracker |
| Точное «3 ч на YouTube» | Да, если считать только foreground |
| Без расширения | Только грубо (title окна) или без URL |

**Функционал жизнеспособен** и ложится в существующую идею продукта: расширение «игровое время» → «время в цифровом пространстве» (игры + браузер) в одном месте.
