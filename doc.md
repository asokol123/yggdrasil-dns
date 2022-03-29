# README

## Конфигурации

На всех машинках запущен yggdrasil и клиенты

На registry ДБ, в базке две таблицы:
1. Сайты
```
{
    'name': string,
    'owner': string,
    'ip': string,
    'expire': uint64,
}
```

2.  Ключи
```
{
    'owner': string,
    'pubkey': string,
}
```


## Запросы
### /register
Запрос:
```
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
 ```
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
```
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

``` 
{
    'address': string,
}
```
