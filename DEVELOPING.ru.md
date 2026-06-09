# Wren — Руководство для разработчиков и продвинутых пользователей

Полная версия документации Wren. В отличие от
[`USAGE.ru.md`](./USAGE.ru.md), ориентированного на обычных
пользователей, здесь есть native-установка, сборка из исходников,
подробная карта файлов и политика polkit.

🇬🇧 English version — [DEVELOPING.md](./DEVELOPING.md).

---

## 1. Установка

### Вариант A — Flatpak bundle (рекомендуется)

Работает на любом Linux с Flatpak: Ubuntu, Fedora, Arch, openSUSE…

```bash
# 1. Добавить Flathub (если ещё не добавлен)
flatpak remote-add --if-not-exists --user flathub \
    https://dl.flathub.org/repo/flathub.flatpakrepo

# 2. Скачать .flatpak с
#    https://github.com/j0ck4/wren-project/releases
#    (файл вида `wren-v0.2.0-dev1.flatpak`)

# 3. Установить
flatpak install --user ~/Downloads/wren-v0.2.0-dev1.flatpak

# 4. Запустить
flatpak run io.github.j0ck4.Wren.Devel
# …или найти «Wren» в меню приложений
```

Первая установка также подтянет GNOME 50 runtime (~600 МБ —
общий для всех Flatpak-приложений).

Дополнительно нужны **WireGuard tools** на хосте (Wren вызывает
`wg-quick`, который не может запускаться внутри Flatpak-песочницы):

```bash
# Debian / Ubuntu
sudo apt install wireguard-tools

# Fedora
sudo dnf install wireguard-tools

# Arch
sudo pacman -S wireguard-tools
```

### Вариант B — Native установка (Ubuntu 24.04+)

```bash
sudo apt install -y meson ninja-build pkg-config \
    libgtk-4-dev libadwaita-1-dev libglib2.0-dev libdbus-1-dev \
    wireguard-tools

# Нужен Rust 1.85+. Если в apt rustc слишком старый — поставьте rustup:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"

git clone https://github.com/j0ck4/wren-project.git
cd wren-project
meson setup builddir --prefix=/usr --buildtype=release
meson compile -C builddir
sudo meson install -C builddir --no-rebuild
```

После установки запускайте **Wren** из меню приложений.

### Вариант C — Сборка через Flatpak SDK

Для разработки. См. [README.md](./README.md).

### Удаление

Wren ставится двумя способами, поэтому удаляйте тем же, каким
ставили. Способы независимы — если пробовали оба, в меню будет два
значка **Wren**, и удалять нужно оба.

**Flatpak** (Вариант A или C):

```bash
flatpak uninstall --user io.github.j0ck4.Wren.Devel
flatpak uninstall --unused          # убрать осиротевший runtime
```

**Native** (Вариант B) — Meson запоминает, что установил, и умеет
откатить из того же каталога сборки:

```bash
cd wren-project
sudo ninja -C builddir uninstall
```

Удаляются ровно те файлы, что Meson положил в `/usr`
(`/usr/bin/wren`, `.desktop`, metainfo, gresource, иконки и политика
polkit).

Обе команды не трогают ваши туннели и настройки. Чтобы стереть и их
— учтите, что в них приватные ключи — удалите каталог конфигов:

```bash
rm -rf ~/.config/wren                                  # native
rm -rf ~/.var/app/io.github.j0ck4.Wren.Devel           # Flatpak
```

---

## 2. Первый запуск

При первом старте Wren откроется пустое окно с единственной кнопкой
**Import .conf**.

Конфигурационный файл WireGuard выглядит так (расширение `.conf`):

```ini
[Interface]
PrivateKey = aGVsbG93b3JsZGhlbGxvd29ybGRoZWxsb3dvcmxkaGVsbA=
Address = 10.0.0.2/32
DNS = 1.1.1.1

[Peer]
PublicKey = cHViMQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=
AllowedIPs = 0.0.0.0/0
Endpoint = vpn.example.com:51820
PersistentKeepalive = 25
```

Этот файл вам выдаёт VPN-провайдер или системный администратор.

---

## 3. Импорт туннеля

1. Нажмите **Import .conf** (большая кнопка на пустой странице или
   иконка папки в шапке боковой панели).
2. Выберите `.conf` файл.
3. Туннель появится в боковой панели.

Wren хранит туннели в `~/.config/wren/tunnels/<имя>.conf` (для
Flatpak — внутри `~/.var/app/io.github.j0ck4.Wren.Devel/config/wren/tunnels/`),
с правами `0600`, чтобы приватный ключ не был доступен другим
пользователям.

Имена WireGuard-интерфейсов ограничены **15 символами**. Если
имя файла длиннее (например, `home-laptop-vpn.conf` — 16 знаков),
Wren автоматически переименует его (`home-laptop-vpn.conf` →
`home-laptop-vpn`).

---

## 4. Подключение / отключение

1. Кликните туннель в боковой панели — справа откроется detail-страница.
2. Нажмите синюю кнопку **Connect** в правом верхнем углу.
3. PolicyKit запросит пароль (один раз на сессию — см. *Polkit policy*).
4. Кнопка станет красной — **Disconnect**. Внизу появится тост
   *<имя> connected* и системное уведомление.

Нажмите **Disconnect** чтобы опустить туннель.

При ошибке тост и уведомление покажут сообщение от `wg-quick`.

---

## 5. Detail-страница туннеля

Когда туннель выбран, справа отображается:

- **Transfer** *(только когда активен)* — Received / Sent в
  KiB / MiB / GiB, обновляется каждые 2 секунды.
- **Interface** — Address, DNS, Listen Port, MTU.
- **Peers** — раскрывающаяся секция на каждый peer. Кликните чтобы
  раскрыть и увидеть публичный ключ, allowed IPs, endpoint,
  keepalive.

Поля subtitle выделяются мышью, можно копировать через ПКМ → Copy
или Ctrl+C.

---

## 6. Редактирование и удаление туннеля

Кликните иконку **карандаш** в шапке detail-страницы.

Откроется модальный диалог с формами:

- **Interface** — Private Key, Address, DNS, Listen Port, MTU.
- **Peers** — раскрывающийся ряд на каждый peer со всеми полями.
  Иконка корзины удаляет peer, кнопка **+** в шапке секции добавляет
  новый peer.

Нажмите **Save** чтобы записать изменения и обновить detail;
**Cancel** — отменить.

Save проверяет, что PrivateKey и PublicKey каждого peer непустые.

Чтобы **удалить** туннель, кликните красную иконку **корзины** в
шапке detail-страницы. Появится диалог подтверждения. Если туннель
активен, Wren сначала опускает его (`wg-quick down` через pkexec), а
затем удаляет `.conf`; если отключение не удалось, файл сохраняется и
показывается toast с ошибкой. Удаление трогает только копию конфига в
каталоге туннелей Wren — ядерный интерфейс не затрагивается, кроме
этого единственного отключения.

---

## 7. Поделиться через QR-код

Чтобы импортировать туннель на телефон:

1. Кликните иконку **QR** (рядом с карандашом) в шапке detail.
2. Отсканируйте QR мобильным приложением WireGuard (Android или iOS):
   *Add tunnel ▸ Create from QR code*.
3. Или нажмите **Copy Configuration** и вставьте конфигурацию
   через *Create from clipboard* в мобильном приложении.

---

## 8. Системный трей

При запуске Wren пытается зарегистрировать трей-иконку
(StatusNotifierItem):

- **KDE / XFCE / Cinnamon / MATE** — иконка появится сама.
- **GNOME** — поставьте расширение [AppIndicator and KStatusNotifierItem
  Support](https://extensions.gnome.org/extension/615/appindicator-support/),
  выйдите из системы и зайдите снова.

ПКМ по иконке открывает меню:

- **Show Wren** — поднять главное окно.
- Список туннелей — клик переключает connect/disconnect.
- **Quit** — закрыть приложение.

Если в окружении нет трея, приложение продолжит работать —
не будет только иконки.

---

## 9. Автозапуск при логине

Кликните **меню ☰** в шапке боковой панели → **Start at Login**.

Это создаст `.desktop` файл в `~/.config/autostart/`, и Wren
будет запускаться автоматически при входе в систему. Снимите
галочку, чтобы удалить запись.

---

## 10. Закрытие окна с активным туннелем

Если закрыть окно при активных туннелях, появится диалог:

- **Cancel** — оставить окно открытым.
- **Disconnect & Quit** — опустить все активные туннели и закрыть
  (может потребовать пароль).
- **Quit Anyway** — закрыть окно; туннели продолжат работать на
  хосте. Опустить их позже можно через
  `sudo wg-quick down <имя>`.

---

## 11. Polkit policy

Wren требует root только чтобы дёргать `wg-quick`. Файл политики
по адресу `/usr/share/polkit-1/actions/io.github.j0ck4.Wren.policy`
содержит `auth_admin_keep`, что значит:

> После первой аутентификации последующие connect/disconnect
> в течение ~5 минут не запрашивают пароль повторно.

Внутри Flatpak файл политики не устанавливается системно, поэтому
промт появляется каждый раз. Используйте Native установку (Вариант B)
или дождитесь `.deb` для Ubuntu 26.04+ для бесшовного процесса.

---

## 12. Решение проблем

**`wg-quick: command not found` при подключении**
Не установлены `wireguard-tools` на хосте. См. *Вариант A* раздела
установки.

**`config file must be a valid interface name, followed by .conf`**
Имя файла содержит недопустимые символы или длиннее 15 знаков. Wren
переименует автоматически при следующем запуске; если нет —
переименуйте файл в `~/.config/wren/tunnels/` так, чтобы он
содержал только `a-z / 0-9 / .-_` и был ≤ 15 символов.

**Иконка трея не видна на GNOME**
Поставьте AppIndicator-расширение (см. *§8*). Wren сам напишет
в логах *Tray service unavailable* и продолжит работать без иконки.

**`pkexec` каждый раз спрашивает пароль**
Вы используете Flatpak-версию. Polkit-policy не может быть
установлена внутри песочницы; используйте Native установку для
кэширования по сессии.

**`Could not refresh tunnel status`**
Wren читает `/sys/class/net` чтобы понять, какие туннели подняты.
На хосте должен быть `ip` (из `iproute2`); он есть на каждом
основном дистрибутиве.

**Туннель поднят, но нет интернета**
Это проблема конфига WireGuard, не Wren. Проверьте, что сервер
доступен, ключ-пара совпадает, а `AllowedIPs` корректные.
`sudo wg show` показывает картину со стороны ядра.

---

## 13. Расположение файлов

| Что | Путь (native) | Путь (Flatpak) |
|-----|---------------|----------------|
| Конфиги туннелей | `~/.config/wren/tunnels/` | `~/.var/app/io.github.j0ck4.Wren.Devel/config/wren/tunnels/` |
| Autostart | `~/.config/autostart/io.github.j0ck4.Wren.desktop` | туда же (через permission `xdg-config/autostart`) |
| Бинарь | `/usr/bin/wren` | внутри bundle |
| Polkit policy | `/usr/share/polkit-1/actions/io.github.j0ck4.Wren.policy` | не устанавливается |

---

## 14. Сообщить о баге

GitHub issues: <https://github.com/j0ck4/wren-project/issues>.

Что приложить к багрепорту:

- Сам `.conf` (с **затёртым** приватным ключом!)
- Вывод `journalctl --user -e | grep wren` или логи терминала
  через `RUST_LOG=wren=debug flatpak run io.github.j0ck4.Wren.Devel`
- `wg-quick --version`, `pkexec --version`, ваш дистрибутив и
  окружение рабочего стола
