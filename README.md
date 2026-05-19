# rip-filter
Simple Rust-based IP filter of incoming connections

# Описание
Простейший IP фильтр входящих соединений. Слушает адрес:порт, проверяет тип соединения с помощью сервиса [proxycheck](https://proxycheck.io), если ответ Residential | Business | Hosting - закрывает соединение отправляя TCP RST, в ином случае делает dial на dest. Кэширует разрешённые IP адреса в RAM Allocated Hash Set. 

# Аргументы
Аргументы запуска
- listen
  Адрес прослушивания входящих соединений в формате `host:port`
- dest
  Конечный адрес в формате `host:port`
- api_key
  Обязательный API ключ сервиса [proxycheck](https://proxycheck.io)
- persistent
  Флаг указывающий, сохранять ли список IP адресов локально в json файле
- mount_path
  Необязательный аргумент пути сохранения лог-файла (по умолчанию `/opt/rip-filter/set.json`)
- poll_period
  Период опроса внешного API (если не указан - значение 15 минут)
- exluded_ip
  Исключённый из фильтрации список IP подсетей в CIDR нотации
