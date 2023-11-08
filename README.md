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
When the setup has been initialized
1. Go to: http://localhost:3005/newpatient
2. Request the following CPR: 0104909995
3. 'Save patient' and it will be available at http://localhost:3005/newpatient


