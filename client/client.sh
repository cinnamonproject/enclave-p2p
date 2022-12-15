curl --cacert ca_bundle.pem -X GET https://ns5019429.ip-15-235-119.net:8100/health

echo "Get block hash based on block number"
curl --cacert ca_bundle.pem -X GET https://ns5019429.ip-15-235-119.net:8100/api/rpcQuery/1112

echo "Get nft data based on nft_id"
curl --cacert ca_bundle.pem -X GET https://ns5019429.ip-15-235-119.net:8100/api/getNFTData/4321

echo "Transfer specified amount of CAPS from Alice to Bob"
curl --cacert ca_bundle.pem -X GET [/https://ns5019429.ip-15-235-119.net:8100/api/submitTx/14983760328739

echo "Send secret shares to TEE"
curl --cacert ca_bundle.pem -X POST https://ns5019429.ip-15-235-119.net:8100/api/nft/storeSecretShares -H 'Content-Type: application/json' -d '{"account_address":"5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", "secret_data": "<Bytes>248_a1b2c3d4e5f60000999988887777</Bytes>", "signature": "0xa6239620eba8e7bc90a6fed86890b7e678f13c7d2a8354bdff705d30b41a294e27c6e2d2d4b7a5fc49d302516580e1a7d25ceaa105c6262c78e1e728e994f38c"}'

echo "Get secret shares from TEE"
curl --cacert ca_bundle.pem -X POST https://ns5019429.ip-15-235-119.net:8100/api/nft/retrieveSecretShares -H 'Content-Type: application/json' -d '{"account_address":"5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy", "secret_data": "<Bytes>248_a1b2c3d4e5f60000999988887777</Bytes>", "signature": "0xa6239620eba8e7bc90a6fed86890b7e678f13c7d2a8354bdff705d30b41a294e27c6e2d2d4b7a5fc49d302516580e1a7d25ceaa105c6262c78e1e728e994f38c"}'
