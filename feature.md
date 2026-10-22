# Feature: Gaming Recap (Игровой Обзор)

Анимированный интерактивный отчёт за месяц / год в стиле Spotify Wrapped.
Пользователь открывает его из навигации или со страницы статистики и
пролистывает серию полноэкранных «слайдов», каждый из которых показывает
одну метрику с числом, подписью и анимацией появления.

---

## Мотивация

Страница статистики уже показывает графики и цифры, но подаёт их утилитарно.
Recap превращает те же данные в эмоциональный, «шэрабельный» опыт:
- геймер видит персональную историю за период;
- каждый слайд — одна метрика, крупная типографика, motion-анимация;
- в конце — сводная карточка-визитка, которую можно сохранить как PNG.

---

## Источники данных (что уже есть)

| Данные | Откуда | API / стор |
|---|---|---|
| Общее время за период | `Statistics.totalPlaytimeSeconds`, `daily[]` | `api.getStatistics()` |
| Топ-5 игр по времени | `Statistics.mostPlayed: RankedPlaytime[]` | `api.getStatistics()` |
| Распределение по жанрам | `Statistics.byGenre: GenreCount[]` | `api.getStatistics()` |
| Кол-во сессий / средняя / самая длинная | `Statistics.sessionsCount`, `avgSessionSeconds`, `longestSessionSeconds` | `api.getStatistics()` |
| Ачивки (получено / всего) | `Statistics.achievementsUnlocked`, `achievementsTotal` | `api.getStatistics()`, `achievementsStore` |
| Кол-во игр / установленных / избранных | `Statistics.gamesCount`, `installedCount`, `favoriteCount` | `api.getStatistics()` |
| Playtime по дням / неделям / месяцам | `Statistics.daily[]`, `weekly[]`, `monthly[]` | `api.getStatistics()` |
| Обложки для топ-игр | `RankedPlaytime.coverPath` | уже в ответе |
| Список всех ачивок (для «самая редкая») | `Achievement[]` через `achievementsStore` | `api.getAchievements()` |

> **Бэкенд-изменения не требуются для MVP.** Все данные агрегируются на фронте
> из уже существующих API-ответов. Если в будущем понадобится фильтрация по
> произвольному date-range — добавим параметры `from`/`to` в команду
> `get_statistics`.

---

## Дизайн слайдов

Recap состоит из **8 слайдов**, каждый занимает весь viewport страницы.
Пользователь переходит между ними кликом, скроллом или стрелками.

### Слайд 1 — Заставка
- Заголовок «Твой игровой {месяц/год}» с анимацией typewriter.
- Фоновый gradient + subtle particle/glow-эффект.
- Кнопка «Начать →».

### Слайд 2 — Общее время
- Крупное число: `totalPlaytimeSeconds`, отформатированное в часы.
- Подпись-сравнение: «Это как пересмотреть Властелина Колец N раз».
- Animated count-up при появлении.

### Слайд 3 — Топ-5 игр
- Пронумерованный вертикальный список с обложками из `RankedPlaytime.coverPath`.
- У каждой — progress-бар относительно первого места.
- Stagger-анимация: карточки влетают одна за другой.

### Слайд 4 — Любимый жанр
- Донат-чарт `byGenre` (уже есть `GenreChart`).
- Крупно выделен топ-1 жанр с процентом от всех игр.
- «Ты — настоящий {RPG / Shooter / …} геймер».

### Слайд 5 — Марафонец (сессии)
- `sessionsCount` — сколько сессий за период.
- `longestSessionSeconds` — самый длинный марафон, с указанием игры.
- `avgSessionSeconds` — средняя длина сессии.
- Иконка таймера с пульсирующей анимацией.

### Слайд 6 — Ачивки
- `achievementsUnlocked / achievementsTotal` с ProgressBar.
- Самая редкая (Legendary) полученная ачивка с названием и описанием.
- Золотой glow при появлении Trophy-иконки.

### Слайд 7 — Хитмапа активности
- Календарь-сетка а-ля GitHub, построенная из `daily[]`.
- Каждая ячейка = день, цвет от прозрачного до accent по `seconds`.
- Подпись: «Активных дней: N», «Лучшая серия: N дней подряд».

### Слайд 8 — Итоговая карточка-визитка
- Компактная summary-карточка с ключевыми числами и топ-3 обложками.
- Кнопки: **«Сохранить PNG»** (html2canvas / dom-to-image) и **«Закрыть»**.

---

## Архитектура

```
src/
├── pages/
│   └── RecapPage.tsx              # роутинг + загрузка данных
├── features/
│   └── recap/
│       ├── RecapSlideshow.tsx      # контейнер слайд-шоу, навигация, progress bar
│       ├── slides/
│       │   ├── IntroSlide.tsx
│       │   ├── TotalPlaytimeSlide.tsx
│       │   ├── TopGamesSlide.tsx
│       │   ├── FavoriteGenreSlide.tsx
│       │   ├── SessionsSlide.tsx
│       │   ├── AchievementsSlide.tsx
│       │   ├── HeatmapSlide.tsx
│       │   └── SummarySlide.tsx
│       ├── components/
│       │   ├── SlideWrapper.tsx    # layout + enter/exit анимация (framer-motion)
│       │   ├── CountUp.tsx         # числовой count-up
│       │   ├── ActivityHeatmap.tsx # SVG-календарь
│       │   └── RecapCard.tsx       # итоговая карточка для экспорта
│       ├── hooks/
│       │   └── useRecapData.ts     # агрегирует данные из сторов для recap
│       └── utils/
│           ├── funFacts.ts         # сравнения и шутливые подписи
│           └── exportImage.ts      # dom → png (html2canvas)
```

---

## Роутинг

```tsx
// App.tsx — добавить роут
<Route path="/recap" element={<RecapPage />} />
```

Точки входа:
- Кнопка на `StatisticsPage` → «Посмотреть Recap»
- Пункт в `CommandPalette` → «Игровой обзор»
- Quick-link на `HomePage`

---

## Ключевой хук `useRecapData`

```ts
interface RecapData {
  period: "month" | "year";
  totalSeconds: number;
  topGames: RankedPlaytime[];       // топ-5 из Statistics.mostPlayed
  topGenre: GenreCount;             // первый элемент byGenre
  genres: GenreCount[];             // весь byGenre
  sessionsCount: number;
  avgSessionSeconds: number;
  longestSessionSeconds: number;
  achievementsUnlocked: number;
  achievementsTotal: number;
  rarestAchievement: Achievement | null;
  heatmap: PlaytimeByDay[];        // daily[]
  activeDays: number;              // дни с seconds > 0
  longestStreak: number;           // подряд дней с seconds > 0
  gamesCount: number;
  favoriteCount: number;
}
```

Хук вызывает `statisticsStore.load()` + `achievementsStore.load()`,
агрегирует и возвращает `RecapData`. Если данных нет — возвращает `null`,
страница показывает empty-state.

---

## Анимация (framer-motion)

- `AnimatePresence` + `mode="wait"` для смены слайдов.
- Каждый `SlideWrapper` принимает `variants` для entry/exit.
- `CountUp` — `useMotionValue` + `useTransform` + `animate` для плавного
  инкремента числа от 0 до target за 1.5 с.
- Stagger-анимация списков через `transition.staggerChildren`.

Библиотека `framer-motion` уже в зависимостях — дополнительных пакетов
не нужно.

---

## Экспорт PNG

Для кнопки «Сохранить» на Summary-слайде:
1. `html2canvas` (или `dom-to-image-more`) рендерит `RecapCard` в canvas.
2. `canvas.toBlob()` → `URL.createObjectURL()`.
3. Через Tauri `saveDialog` + `plugin-fs` записываем файл.

Потребуется добавить одну dev-зависимость: `html2canvas` (≈ 40 KB gzip).

---

## i18n

Добавить ключи в `src/i18n/index.ts`:

```
recap.title           → "Игровой обзор"
recap.month           → "Твой игровой месяц"
recap.year            → "Твой игровой год"
recap.totalTime        → "Суммарно наиграно"
recap.topGames         → "Топ-5 игр"
recap.favoriteGenre    → "Любимый жанр"
recap.sessions         → "Сессий сыграно"
recap.longestSession   → "Самый длинный марафон"
recap.avgSession       → "Средняя сессия"
recap.achievements     → "Ачивки получены"
recap.rarestAch        → "Самая редкая ачивка"
recap.activeDays       → "Активных дней"
recap.longestStreak    → "Лучшая серия"
recap.save             → "Сохранить как картинку"
recap.start            → "Начать"
recap.next             → "Далее"
recap.close            → "Закрыть"
```

---

## Дополнительные идеи (после MVP)

- **Выбор периода**: dropdown «Этот месяц / Прошлый месяц / Этот год».
  Потребуется бэкенд: параметры `from`/`to` в `get_statistics`.
- **Авто-показ**: при первом открытии приложения в начале нового месяца —
  предложить посмотреть Recap за прошлый.
- **Звуковые эффекты**: тонкий ambient при переключении слайдов
  (опционально в настройках).
- **Сравнение**: «В этом месяце ты наиграл на 12 ч больше, чем в прошлом».

---

## Этапы реализации

| # | Этап | Описание |
|---|---|---|
| 1 | Данные | `useRecapData` хук + `funFacts.ts` |
| 2 | Каркас | `RecapPage`, `RecapSlideshow`, `SlideWrapper`, навигация |
| 3 | Слайды | Реализовать 8 слайдов по одному |
| 4 | Анимации | count-up, stagger, enter/exit transitions |
| 5 | Хитмапа | `ActivityHeatmap` SVG-компонент |
| 6 | Экспорт | Итоговая карточка + сохранение PNG |
| 7 | Интеграция | Роут, ссылки с HomePage / StatisticsPage / CommandPalette |
| 8 | i18n | Ключи для ru/en |
