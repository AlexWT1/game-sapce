# AGENTS.MD — Game Space

## Предназначение
Game Space — кросс-платформенный десктопный лаунчер игр (Windows / macOS / Linux): библиотека игр,
запуск, мониторинг процессов и подсчёт времени игры, достижения, статистика, темы/акценты,
системный трей, онбординг, i18n (ru/en). UI — тёмный, «премиальный» (glassmorphism).
Продуктовый спек лежит локально в `assets/project.md` (файл в .gitignore, в репозиторий не попадает).

## Стек
- **Backend**: Tauri 2 + Rust 2021. `rusqlite` (bundled SQLite, WAL), `thiserror`/`anyhow`,
  `sysinfo`, `walkdir`, `chrono`, `log` + самописный FileLogger; фичи `tray-icon`, `image-png`;
  плагины: `tauri-plugin-dialog`, `tauri-plugin-single-instance`.
- **Frontend**: React 19, TypeScript 5.8 (strict, `noUncheckedIndexedAccess`), Vite 6,
  Tailwind CSS 4, Zustand (+persist), Framer Motion, Recharts, Lucide icons,
  react-router-dom (HashRouter — обязателен для file:// origin в Tauri), cva/clx.
- **i18n**: словари ru/en в `src/i18n/index.ts`; тип `Dict = typeof RU`, поэтому EN обязан
  содержать все ключи RU (иначе ошибка типов).

## Структура
- `src-tauri/src/` — Rust: `lib.rs` (Builder: плагины → setup: logger → БД → seed → tray →
  монитор; `on_window_event` (drag&drop); `invoke_handler`), `commands/{games,launch,achievements,settings,stats,system}`,
  `database/`, `game/{launcher,achievements,seed,theme}`, `process/monitor`, `tray/`,
  `services/{discovery,export}`, `models` (все DTO с `#[serde(rename_all="camelCase")]`),
  `error` (`AppError` сериализуется фронтенду как plain string).
- `src/` — React: `pages/` → `features/` → `components/{domain,ui,layout}`; кросс-модульное:
  `stores/`, `services/`, `hooks/`, `utils/`, `types/`, `i18n/`.
- Миграции: `migrations/*.sql` (вшиты через `include_str!`, только forward-compatible).

## Ключевые особенности
- Идентификатор приложения: `com.gamespace.launcher`; окно 1440x900, `decorations: false`
  (кастомный titlebar с WindowControls), `center: true`.
- Кнопка «Закрыть» НЕ завершает процесс — окно прячется в трей (`window_close` → `hide`);
  выход только через меню трея (`app.exit(0)`). Повторный запуск бинарника активирует
  существующий экземпляр (single-instance), дубликат иконки в трее невозможен.
- Данные: `<app_data>/game-space.db`, логи `<app_data>/logs/game-space.log`.
- Настройки хранятся ДВАЖДЫ: localStorage (`gs-settings`, Zustand persist) + таблица `settings`
  (key/value, generic команды `get_settings`/`update_settings`/`get_setting`).
  Рабочие ключи: `demo_mode`, `monitoring_interval`, `default_games_dir`, `auto_detect`, `language`…
- Tauri-события (подписка в `src/services/events.ts` / `useAppEvents`):
  `game-started {gameId,pid,title}` | `game-started {recovered:true}` | `game-stopped {gameId,durationSeconds}`
  | `game-deleted {gameId}` | `achievement-unlocked` | `statistics-updated {}` | `navigate {path}`
  | `file-dropped {paths}`. Событий game created/updated и settings changed НЕТ — фронт перезапрашивает данные.
- Роуты (HashRouter, всё внутри AppLayout): `/`, `/library` (query: filter/genre/developer),
  `/game/:id`, `/achievements`, `/statistics`, `/friends`, `/shop`, `/settings`;
  онбординг — не роут, а ветка в AppLayout (`settings.onboardingDone === false`).
- Поиск игр: команда `discover_games` (walkdir, depth 4, cap 200) с чёрными списками папок/имён
  exe (редисты, Engine, DirectX, uninstallers, CrashHandler, программы-лаунчеры);
  общий фронт-сервис `src/services/discovery.ts` → `scanForGames(dirs)` используется онбордингом,
  кнопкой в настройках и автосканом при старте (`auto_detect` + `default_games_dir`).

## Конвенции фронтенда
- Только named exports (исключение — default export `App`), файлы `PascalCase.tsx` / `camelCase.ts`,
  хуки `useX`, сторы `useXStore`, импорты через `@/` (относительные — только в `main.tsx`),
  `import type` для типов.
- Никакого `any`; `slice()` перед `.sort()` (не мутировать массивы сторов);
  `void` для fire-and-forget; `loading="lazy"` на изображениях; тосты: error 6s, остальные 4s.
- Эмодзи как иконки запрещены — только Lucide. Без «фиктивных» фич вне MVP-скоупа.
- В Rust production-путях не использовать `unwrap()`; ошибки — дружелюбные RU-тосты,
  стектрейсы Rust пользователю не показывать (логировать).
- Фронт никогда не ходит напрямую в SQLite и не исполняет shell — только через команды Tauri.
- Playtime хранится в секундах.

## Проверки после нетривиальных изменений (обязательный триплет)
1. `npm run typecheck`
2. `npm run build`
3. `cd src-tauri && cargo check` (0 warnings)
4. Если менялся Rust-код с логикой: `cd src-tauri && cargo test --lib`
