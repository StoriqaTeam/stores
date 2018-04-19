#!/bin/sh

echo "Waiting for kafka-connect"
sleep 40

echo "Initializing connectors"
KAFKA_CONNECT_ADDR=$1

sleep 5
cat << EOF > pg-connector.json
{
  "name": "stores-pg-connector",
  "config": {
    "connector.class": "io.debezium.connector.postgresql.PostgresConnector",
    "database.user": "stores",
    "database.dbname": "stores",
    "database.hostname": "stores-pg",
    "database.password": "stores",
    "name": "stores-pg-connector",
    "database.server.name": "stores-pg",
    "database.port": "5432",
    "transforms": "Reroute",
    "transforms.Reroute.type": "io.debezium.transforms.ByLogicalTableRouter",
    "transforms.Reroute.topic.regex": ".*",
    "transforms.Reroute.topic.replacement": "stores-pg",
    "transforms.Reroute.key.field.name": "table",
    "transforms.Reroute.key.field.regex": "(.*)\\\.(.*)\\\.(.*)",
    "transforms.Reroute.key.field.replacement": "\$3"
  }
}
EOF
curl -X POST -H "Accept:application/json" -H "Content-Type:application/json"  -d @pg-connector.json $KAFKA_CONNECT_ADDR/connectors

sleep 5
cat << EOF > stores-connector.json
{
  "name": "stores-connector",
  "config": {
    "connector.class": "com.skynyrd.kafka.ElasticSinkConnector",
    "topics": "stores-pg",
    "tasks.max": "1",
    "type.name": "_doc",
    "elastic.url": "stores-es",
    "elastic.port": "9200",
    "index.name": "products"
  }
}
EOF
curl -X POST -H "Content-Type: application/json" -H "Accept: application/json" -d @stores-connector.json $KAFKA_CONNECT_ADDR/connectors

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
               "rating": {
                  "type": "double"
               },
               "country": {
                  "type": "text"
               },
               "id": {
                  "type": "integer"
               },
               "product_categories": {
                  "type": "nested",
                  "properties": {
                     "category_id": {
                        "type": "integer"
                     },
                     "count": {
                        "type": "integer"
                     }
                  }
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
               "views": {
                  "type": "integer"
               },
               "variants": {
                  "type": "nested",
                  "properties": {
                     "prod_id": {
                        "type": "integer"
                     },
                     "discount": {
                        "type": "double"
                     },
                     "price": {
                        "type": "double"
                     },
                     "attrs": {
                        "type": "nested",
                        "properties": {
                          "attr_id": {
                              "type": "integer"
                          },
                          "float_val": {
                              "type": "double"
                          },
                          "str_val": {
                              "type": "text"
                          }
                        }
                     }
                  }
               },
               "suggest" : {
                   "type" : "completion"
               }
            }
         }
      }
}
'
