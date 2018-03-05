# stores

Microservice for managing store profiles.

## Running

```
cd docker && docker-compose up
```

For ElasticSearch to work it's necessary to put kc-plugins folder from https://github.com/StoriqaTeam/kafka-elastic-sink-connector/tree/master repo under docker/kafka_connect in this repo

## Request Flow

* `Application` ⇄ `Router` ⇄ `Service` ⇄ `Repo`

## API Documentation

* [Postman Documenter](https://documenter.getpostman.com/view/131444/users/7LjD5Hc)
