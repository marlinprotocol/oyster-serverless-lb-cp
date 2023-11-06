# oyster-serverless-lb-cp

This is the Control Plane for the Oyster Serverless Load Balancer. It is used to add and remove backend servers from the load balancer configuration.

# Endpoints

## add-server

Example CURL -
```shell
curl -X POST localhost:8012/add-server -d '{"ip": "127.03.05.85:8534", "capacity": 12000}' -H "Content-type: application/json" -H "Accept: application/json"
```

Here, the capacity is the capacity of the backend server in MBs.

## remove-server

Example CURL -
```shell
curl -X POST localhost:8012/remove-server -d '{"ip": "127.03.05.85:846"}' -H "Content-type: application/json" -H "Accept: application/json"
```
