# Fin-Dashboard
It is a web/websocket server that provides real-time data.
default run on port 8080 and waiting for data from worker

## workers
worker is a bot to get information and send it to the server. 
look at folder `workers`

## test websocket client
```sh 
websocat ws://127.0.0.1:8080/ws
```

## the concept 
1. start the server
```sh
cargo run 
```
2. start bot around 5 machine 
```sh
// 1 machine 
cd workers
cargo run 
```
3. open brownser and visit http://localhost:8080/dashboard

or run
```sh
run_all.sh
```
