# README

## Конфигурации

На всех машинках запущен yggdrasil и клиенты

На registry ДБ, в базке две таблицы:
1. Сайты
```json
{
    'name': string,
    'owner': string,
    'ip': string,
    'expire': uint64,
}
```

2.  Ключи
```json
{
    'owner': string,
    'pubkey': string,
}
```


## Запросы
### /register
Запрос:
```json
{
    'name': string,
    'pubkey': string,
    'timestamp': uint64,
    'nonce': string
}
```

 Схема проверки:
 1. Проверяется proof of work
 
 ### /set_site
 Запрос:
 ```json
{
    'site': string,
    'address': string,
    'expire': string,
    'owner': string,
    'signature': string,
    'timestamp': uint64,
    'nonce': string
}
```

 Схема проверки:
 1. Проверяется proof of work
 2. Проверяется пользователь и подпись
 3. Проверяется expire
 4. Проверяется, что сайта нет либо он создан тем же пользователем

### /get_site
Запрос:
```json
{
    'site': string,
    'timestamp': uint64,
    'nonce': string
}
```

 Схема проверки:
 1. Проверяется proof of work
 2. Проверяется expire
 Ответ:

```json 
{
    'address': string,
}
```
