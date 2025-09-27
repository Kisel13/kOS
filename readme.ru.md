# k/OS

> Languages: [English](README.md) | Russian

###### Форк Blog OS, операционной системы из серии [Writing an OS in Rust](https://os.phil-opp.com). Смотрите [LICENSE].

[LICENSE]: LICENSE
[LICENSE-GPL]: LICENSE-GPL

## Сборка

Для некоторых unstable функций нужна nightly версия Rust. Запустите `rustup update nightly --force` чтобы обновиться на последнюю nightly сборку.

Чтобы собрать просто запустите `cargo build`.

Для создания загрузочного диска нужен инструмент [`bootimage`]:

[`bootimage`]: https://github.com/rust-osdev/bootimage

```
cargo install bootimage
```

После установки, создайте загрузочный диск командой:

```
cargo bootimage
```

Создаётся загрузочный образ диска в `target/x86_64-kos/debug/bootimage-kos.bin`.

## Запуск

Образ диска можно загрузить почти в любой x86_64 эмулятор.

`cargo run` запускает виртуальную машину [QEMU].

Для этого нужно установить [QEMU] и [`bootimage`].

[QEMU]: https://www.qemu.org/

Также можно записать образ на реальный носитель и загрузить систему на реальной машине. На Linux, используйте команду:

```
sudo dd if=target/x86_64-kos/debug/bootimage-kos.bin of=/dev/sdX conv=fsync status=progress
```

Где `sdX` это имя носителя. 

> **Вся информация на устройстве будет перезаписана!**

## Тестирование

Для запуска тестов используйте `cargo test`.

## Лицензия

Этот форк распространяется под лицензией GNU General Public License, версии 3

Вы должны были получить копию GNU General Public License (на английском языке)
вместе с программой. если нет, смотрите <https://www.gnu.org/licenses/>.

Читайте [LICENSE] и [LICENSE-GPL]

## Состояние

- Ядро
  - [X] Драйвер текстового буфера VGA
  - [X] Исключения процессора
  - [X] Управление памятью, аллокатор.
  - [X] Драйвер serial шины.
  - [X] PS/2 клавиатура
  - [ ] Многозадачность
    - [X] Реализация Async/Await
