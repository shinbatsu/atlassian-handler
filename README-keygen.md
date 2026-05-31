# 🦀 Atlassian Keygen — Rust CLI

Генератор лицензионных ключей для продуктов Atlassian (Jira, Confluence, Crowd, Bitbucket и др.), написанный на Rust.

## 🔧 Требования

- [Rust](https://www.rust-lang.org/tools/install) 1.70+
- Стандартный `cargo`

## 📦 Сборка

```bash
cd rust-keygen

# Простая сборка
cargo build --release

# Или через build.sh
./build.sh linux      # Linux x86_64
./build.sh windows    # Windows x86_64
./build.sh macos      # macOS x86_64
./build.sh all        # Все платформы
```

Готовый бинарник: `target/release/handler`

## 🚀 Использование

```bash
# Базовая команда (Server)
./handler -m admin@example.com -o "My Company" -p crowd -s ABCD-1234-EFGH-5678

# Data Center
./handler -m admin@example.com -n "Admin" -o "My Company" -p jira -s ABCD-1234-EFGH-5678 -d

# С указанием имени (отдельно от email)
./handler -m admin@example.com -n "Admin User" -o "My Org" -p conf -s ABCD-1234-EFGH-5678
```

### Флаги

| Флаг             | Описание                             |
| ---------------- | ------------------------------------ |
| `-h`             | Справка                              |
| `-m <email>`     | Email лицензии (обязательно)         |
| `-n <name>`      | Имя владельца (по умолчанию = email) |
| `-o <org>`       | Организация (обязательно)            |
| `-p <product>`   | Продукт (обязательно)                |
| `-s <server-id>` | Server ID (обязательно)              |
| `-d`             | Data Center лицензия                 |

## 📋 Поддерживаемые продукты

| Ключ        | Продукт                              |
| ----------- | ------------------------------------ |
| `crowd`     | Crowd                                |
| `jira`      | JIRA Software                        |
| `conf`      | Confluence                           |
| `bitbucket` | Bitbucket                            |
| `bamboo`    | Bamboo                               |
| `fisheye`   | FishEye                              |
| `crucible`  | Crucible                             |
| `jsm`       | JIRA Service Management              |
| `jc`        | JIRA Core                            |
| `jsd`       | JIRA Service Desk                    |
| `questions` | Questions plugin for Confluence      |
| `capture`   | Capture plugin for JIRA              |
| `training`  | Training plugin for JIRA             |
| `portfolio` | Portfolio plugin for JIRA            |
| `tc`        | Team Calendars plugin for Confluence |

## 🔄 Алгоритм работы

1. **Формирование данных лицензии** — создаётся набор ключ-значение с параметрами (даты, контакты, версия, тип)
2. **Вычисление licenseHash** — SHA-256 от отсортированных свойств с экранированием спецсимволов
3. **Формирование текста** — `#<timestamp>\n<key>=<value>\n...`
4. **Сжатие** — zlib (Deflate)
5. **Подпись** — DSA-1024 с SHA-1
6. **Упаковка** — `[4-байт длина][данные][DSA-подпись]`
7. **Base64 + суффикс** — `base64(data) + "X02" + hex(длина)`
8. **Разбивка** — по 76 символов в строке

## 🔑 DSA-ключ

Используется DSA-ключ из оригинального atlassian-agent.jar:
- P: 1024 бита
- Q: 160 бит
- G: 1024 бита
- X: 160 бит (закрытый ключ)