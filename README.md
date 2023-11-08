# Hjemmebehandling-compose
A compose setup for initializing the whole project including all the services.

Two frontends will be available at 3005 and 3006, respectively "infektionsmedicinsk" and "lungemedicinsk".

## Getting Started

Start the solution
```
cd ./compose
docker-compose up
```

Stop the solution
```
docker-compose down
```

In case of corrupted data or the need of restarting the setup run the following:
```
docker-compose down
docker system prune --volumes
docker-compose up
```


