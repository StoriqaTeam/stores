#!/bin/sh

echo "Waiting for kafka-connect"
sleep 120

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
    "type.name": "_doc",
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
    "topics": "stores-pg.public.base_products,stores-pg.public.prod_attr_values",
    "tasks.max": "1",
    "type.name": "_doc",
    "elastic.url": "stores-es",
    "index.name": "products",
    "elastic.port": "9200"
  }
}
EOF
curl -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d @products-connector.json $KAFKA_CONNECT_ADDR/connectors


sleep 5
echo "Initializing Elastic indices"

curl -XPUT 'stores-es:9200/stores?pretty' -H 'Content-Type: application/json' -d'
{
      "mappings": {
         "_doc": {
            "properties": {
               "name": {
                  "type": "nested",
                  "properties": {
                     "lang": {
                        "type": "text"
                     },
                     "text": {
                        "type": "text"
                     }
                  }
               },
               "user_id": {
                  "type": "integer"
               },
               "id": {
                  "type": "integer"
               },
               "suggest" : {
                   "type" : "completion"
               }
            }
         }
      }
}
'

sleep 5

curl -XPUT 'stores-es:9200/products?pretty' -H 'Content-Type: application/json' -d'
{
      "mappings": {
         "_doc": {
            "properties": {
               "name": {
                  "type": "nested",
                  "properties": {
                     "lang": {
                        "type": "text"
                     },
                     "text": {
                        "type": "text"
                     }
                  }
               },
               "short_description": {
                  "type": "nested",
                  "properties": {
                     "lang": {
                        "type": "text"
                     },
                     "text": {
                        "type": "text"
                     }
                  }
               },
               "long_description": {
                  "type": "nested",
                  "properties": {
                     "lang": {
                        "type": "text"
                     },
                     "text": {
                        "type": "text"
                     }
                  }
               },
               "id": {
                  "type": "integer"
               },
               "category_id": {
                  "type": "integer"
               },
               "variants": {
                  "type": "nested"
               },
               "suggest" : {
                   "type" : "completion"
               }
            }
         }
      }
}
'
