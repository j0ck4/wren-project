# Wren — WireGuard GUI для Ubuntu

> Нативный GTK4 / libadwaita клиент для управления WireGuard туннелями.  
> Язык: Rust | Платформы: Ubuntu 24.04 LTS, 26.04 LTS

🇬🇧 English version — [wren-project.md](./wren-project.md).

---

## Контекст

На данный момент не существует нативного GTK4/libadwaita клиента WireGuard для Linux. Ближайший аналог — **Wrenrd** (Go + GTK3, ~960 ⭐), последнее обновление январь 2024, не поддерживает Ubuntu 24.04+. Остальные проекты либо заброшены, либо используют веб-обёртки (Tauri + Electron).

Данный проект занимает пустую нишу и следует современному стеку GNOME.

---

## Цели проекта

- Предоставить нативный, визуально современный WireGuard клиент для Ubuntu
- Поддерживать Ubuntu 24.04 LTS и 26.04 LTS из коробки
- Распространяться как `.deb` пакет и Flatpak
- Не требовать постоянного ввода пароля (polkit / pkexec)

---

## Стек технологий

| Компонент       | Выбор                        | Обоснование                              |
|-----------------|------------------------------|------------------------------------------|
| Язык            | Rust                         | Безопасность, производительность, Ubuntu 26.04 |
| UI фреймворк    | gtk4-rs + libadwaita         | Нативный GNOME стек                      |
| Async runtime   | tokio                        | Фоновые задачи, опрос статистики         |
| Системный трей  | ksni                         | StatusNotifierItem (современный стандарт)|
| Сборка UI       | Blueprint (опционально)      | Декларативный XML для GTK4               |
| Привилегии      | polkit / pkexec              | Стандарт Ubuntu для прав администратора  |
| Хранение данных | glib::user_config_dir        | `~/.config/wren/`                      |
| Парсинг .conf   | ini crate + serde            | Формат WireGuard INI-подобный            |

---

## Функциональные требования

### MVP (v0.1)

- [ ] Импорт `.conf` файла через диалог открытия файла
- [ ] Список туннелей в боковой панели (sidebar)
- [ ] Подключение / отключение туннеля одной кнопкой
- [ ] Статус туннеля (подключён / отключён / ошибка)
- [ ] Системный трей с индикатором состояния и меню

### v0.2

- [ ] Просмотр деталей туннеля (IP, DNS, peers, allowed IPs)
- [ ] Статистика трафика (RX / TX) с обновлением раз в 2 сек
- [ ] Уведомления GNOME при подключении / отключении
- [ ] Автозапуск при входе в систему

### v0.3

- [ ] Управление несколькими пирами (peer list)
- [ ] Редактирование конфигурации туннеля в UI
- [ ] Экспорт конфига / QR-код для мобильных устройств
- [ ] Flatpak пакет

---

## Нефункциональные требования

- Время запуска приложения: < 300 мс
- Потребление памяти в фоне (трей): < 20 МБ
- Нет зависимостей от libssl, Node.js, Python в runtime
- Поддержка Wayland и X11
- Соответствие GNOME HIG (Human Interface Guidelines)

---

## Архитектура проекта

```
wren/
├── src/
│   ├── main.rs                 # Точка входа, GTK Application
│   ├── app.rs                  # ApplicationWindow, глобальное состояние
│   ├── ui/
│   │   ├── window.rs           # Главное окно (AdwApplicationWindow)
│   │   ├── tunnel_list.rs      # Боковая панель со списком туннелей
│   │   ├── tunnel_detail.rs    # Детали выбранного туннеля + статистика
│   │   ├── import_dialog.rs    # Диалог импорта .conf файла
│   │   └── tray.rs             # Системный трей (ksni)
│   ├── wg/
│   │   ├── manager.rs          # wg-quick up/down через pkexec
│   │   ├── parser.rs           # Парсинг .conf файлов
│   │   ├── stats.rs            # Чтение статистики (wg show transfer)
│   │   └── monitor.rs          # Фоновый опрос статуса туннелей
│   └── models/
│       ├── tunnel.rs           # Структуры Tunnel, Peer, TunnelStatus
│       └── config.rs           # Сохранение/загрузка настроек приложения
├── data/
│   ├── io.github.wren.desktop        # .desktop файл
│   ├── io.github.wren.metainfo.xml   # AppStream метаданные
│   ├── io.github.wren.gschema.xml    # GSettings схема
│   └── icons/
│       ├── hicolor/scalable/apps/wren.svg
│       └── hicolor/symbolic/apps/wren-symbolic.svg
├── polkit/
│   └── io.github.wren.policy         # Polkit политика для wg-quick
├── packaging/
│   ├── debian/                          # Пакет .deb
│   └── io.github.wren.yml            # Flatpak манифест
└── Cargo.toml
```

---

## Ключевые зависимости (Cargo.toml)

```toml
[dependencies]
gtk          = { version = "0.9",  package = "gtk4",       features = ["v4_10"] }
adw          = { version = "0.7",  package = "libadwaita", features = ["v1_4"]  }
ksni         = "0.2"
tokio        = { version = "1",    features = ["full"] }
serde        = { version = "1",    features = ["derive"] }
toml         = "0.8"
ini          = "1.3"
anyhow       = "1"
once_cell    = "1"
glib         = "0.20"
```

---

## Взаимодействие с WireGuard

Приложение управляет туннелями через системные инструменты, не напрямую через ядро:

```
UI Action          →  pkexec wg-quick up   <interface>   # поднять туннель
UI Action          →  pkexec wg-quick down <interface>   # опустить туннель
Background thread  →  wg show <interface> transfer       # статистика RX/TX
Background thread  →  wg show interfaces                 # список активных
```

Конфигурационные файлы при импорте копируются в `/etc/wireguard/` через pkexec.

---

## Polkit политика

Файл `/usr/share/polkit-1/actions/io.github.wren.policy` разрешает запуск `wg-quick` без постоянного запроса пароля для пользователей группы `sudo`:

```xml
<action id="io.github.wren.manage-tunnels">
  <description>Manage WireGuard tunnels</description>
  <defaults>
    <allow_active>auth_admin_keep</allow_active>
  </defaults>
  <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/wg-quick</annotate>
</action>
```

---

## Дорожная карта

| Этап  | Версия | Срок      | Содержание                                      |
|-------|--------|-----------|-------------------------------------------------|
| 1     | v0.1   | 4 недели  | MVP: импорт, список, connect/disconnect, трей   |
| 2     | v0.2   | +3 недели | Статистика, уведомления, автозапуск             |
| 3     | v0.3   | +4 недели | Редактор конфига, multi-peer, Flatpak           |
| 4     | v1.0   | +2 недели | Полировка UI, тесты, публикация на GitHub       |

---

## Конкурентный анализ

| Проект            | Язык     | UI        | Ubuntu 24.04 | Активен | Статистика |
|-------------------|----------|-----------|:------------:|:-------:|:----------:|
| Wrenrd         | Go       | GTK3      | ⚠️ нет      | ❌      | ❌         |
| wireguard-gui     | JS+Tauri | WebView   | ✅           | ⚠️      | ❌         |
| Wren           | ?        | ?         | ❌           | ❌      | ❌         |
| **wren (наш)**  | **Rust** | **GTK4**  | **✅**       | **✅**  | **✅**     |

---

## Системные зависимости (runtime)

```
wireguard-tools   # wg, wg-quick
resolvconf        # управление DNS
polkit            # запрос привилегий
libgtk-4-1        # GTK4
libadwaita-1-0    # Adwaita виджеты
```

---

*Документ создан: май 2026*
