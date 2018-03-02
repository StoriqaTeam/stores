#!/bin/sh

echo "Waiting for kafka-connect"
sleep 60

echo "Initializing connectors"
KAFKA_CONNECT_ADDR=$1

curl -X POST -H "Accept:application/json" -H "Content-Type:application/json" $KAFKA_CONNECT_ADDR/connectors/ -d '{ "name": "stores-pg-connector", "config": { "connector.class": "io.debezium.connector.postgresql.PostgresConnector", "database.user": "stores", "database.dbname": "stores", "database.hostname": "stores-pg", "database.password": "stores", "name": "stores-pg-connector", "database.server.name": "stores-pg", "database.port": "5432" } }'

sleep 5
cat << EOF > stores-connector.json
{
  "name": "stores-connector",
  "config": {
    "connector.class": "com.skynyrd.kafka.ElasticSinkConnector",
    "topics": "stores-pg.public.stores",
    "tasks.max": "1",
    "type.name": "stores-pg",
    "elastic.url": "stores-es",
    "index.name": "stores",
    "elastic.port": "9200"
  }
}
EOF
curl -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d @stores-connector.json $KAFKA_CONNECT_ADDR/connectors

sleep 5
cat << EOF > products-connector.json
{
  "name": "products-connector",
  "config": {
    "connector.class": "com.skynyrd.kafka.ElasticSinkConnector",
    "topics": "stores-pg.public.products",
    "tasks.max": "1",
    "type.name": "stores-pg",
    "elastic.url": "stores-es",
    "index.name": "products",
    "elastic.port": "9200"
  }
}
EOF
curl -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d @products-connector.json $KAFKA_CONNECT_ADDR/connectors

sleep 5
cat << EOF > prod_attr_values-connector.json
{
  "name": "prod_attr_values-connector",
  "config": {
    "connector.class": "com.skynyrd.kafka.ElasticSinkConnector",
    "topics": "stores-pg.public.prod_attr_values",
    "tasks.max": "1",
    "type.name": "stores-pg",
    "elastic.url": "stores-es",
    "index.name": "products",
    "elastic.port": "9200"
  }
}
EOF
curl -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d @prod_attr_values-connector.json $KAFKA_CONNECT_ADDR/connectors