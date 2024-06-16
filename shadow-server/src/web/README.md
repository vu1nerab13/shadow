# Shadow Web API

A restful server which provide a set of function to control `Shadow Server` and `Shadow Client`

*"work in progress..."*

## Client

### Get Summary

`Method`: GET

`Url`: /v1/client/{client_addr}/query?op=summary

`Return`: TODO

### Get Apps

`Method`: POST

`Url`: /v1/client/{client_addr}/app

`Body`: {"op":"enumerate"}

`Return`: TODO

### Get Displays

`Method`: POST

`Url`: /v1/client/{client_addr}/displays

`Body`: {"op":"enumerate"}

`Return`: TODO

### Get Processes

`Method`: POST

`Url`: /v1/client/{client_addr}/process

`Body`: {"op":"enumerate"}

`Return`: TODO

### Kill Process

`Method`: POST

`Url`: /v1/client/{client_addr}/process

`Body`: {"op":"kill","pid":{pid_in_integer}}

### Sleep

`Method`: POST

`Url`: /v1/client/{client_addr}/power

`Body`: {"op":"sleep"}

`Return`: TODO

## Server

### Query Clients

`Method`: GET

`Url`: /v1/server/query?op=clients

`Return`: A array that contains clients' address
