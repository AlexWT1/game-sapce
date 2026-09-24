# Game Space — команды окружения

## Системные зависимости (Linux, Tauri)

```bash
sudo apt-get install -y libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev libssl-dev pkg-config
```

## Rust (если ещё не установлен)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
source "$HOME/.cargo/env"
```

## Frontend

```bash
npm install
npm run dev        # dev-сервер Vite (порт 1420)
npm run build      # tsc + vite build
npm run typecheck  # только проверка типов
```

## Desktop-приложение

```bash
npm run tauri dev     # запуск в dev-режиме
npm run tauri build   # production-сборка
```

## Проверка Rust backend

```bash
cd src-tauri && cargo check
```
