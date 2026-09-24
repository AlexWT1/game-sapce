# Game Space — прогресс

Статус: **процессы остановлены** (кроме старых release-бинарей). Исправлены 3 бага из тестирования: видимые тумблеры, рабочие «Анимации»/«Прозрачность»/«Демо-данные», плавный sidebar. `npm run typecheck` / `npm run build` / `cargo check` — OK.

## Готово

### Phase 1 — foundation
- Rust, Tauri CLI, frontend deps, иконки, конфиги (vite/ts/tauri/capabilities)

### Backend (Rust)
- SQLite/WAL, все команды, launcher, monitor, achievements, demo-seed, tray, drag&drop, logger

### Frontend
- i18n ru/en, services/api (camelCase), stores (zustand), UI-kit, Layout, 8 страниц, HashRouter

### White/black-screen fix
- `index.html` + `src/index.css` — hardcoded `#070912`
- `src/components/ErrorBoundary.tsx` + wrap в `main.tsx`
- Onboarding/Hero/Modal/CommandPalette — `initial={{ opacity: 1 }}`
- Onboarding card — solid `#101326` (без glass/backdrop)
- Welcome title — без `text-gradient` (иначе прозрачный текст)

### Linux dialog fix (выбор папки «ничего не происходит»)
- Причина: Rust `blocking_pick_folder` (GTK) не открывался из command thread
- Решение: `@tauri-apps/plugin-dialog` (`open`/`save`) в `src/services/api.ts`
- Onboarding/Settings/GameForm — try/catch + toast
- capabilities: `dialog:allow-open`, `dialog:allow-save`

### UI bugfix (тестирование пользователя)
1. **Toggle** (`src/components/ui/Toggle.tsx`): видимый трек (`#1a1e35` + border), accent ON + glow, title ON/OFF
2. **Анимации / Прозрачность**: `applyVisualPreferences` → `html[data-animations]` / `html[data-transparency]` + CSS kill switch; `MotionConfig reducedMotion` в `App.tsx`
3. **Демо-данные**: `demoMode` в settingsStore; mock friends → `[]` при off; backend `source=Demo` + фильтр в `get_games`/`stats`/`achievements`/`search`; seed → `demo_seeded` (не ломает toggle); миграция legacy Manual→Demo
4. **Sidebar**: `duration-300 ease-in-out` + плавное скрытие блоков (logo/nav/filters/promo)

### Проверки
- `npm run typecheck` → OK
- `npm run build` → OK (manualChunks: react/motion/charts/tauri/icons)
- `cargo check` → OK (0 warnings; удалены unused GameSource, missing_game, unused chrono imports)

## Осталось (polish)
1. Smoke-test на Linux: onboarding → «Выбрать папку» → диалог; settings toggles; demo off/on; sidebar collapse
2. Theme editor UI

## Полезно знать
- Dev: `npm run tauri dev` (порт 1420)
- Tauri invoke/ответы — camelCase
- Диалоги — только через `@tauri-apps/plugin-dialog`, не Rust blocking
- `rg` недоступен → `grep`
- Не делай широкий `pkill -f` — kill по PID / `killall game-space`
- Остановка: `killall -9 game-space; ps -eo pid,cmd | grep -E 'game-space|bin/vite' | grep -v grep | awk '{print $1}' | xargs -r kill -9`
- Данные: `~/.local/share/com.gamespace.launcher/`
