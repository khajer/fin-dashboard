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
