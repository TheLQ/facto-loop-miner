echo $1 | cut -c2- - | base64 -d | pigz -cd | jq