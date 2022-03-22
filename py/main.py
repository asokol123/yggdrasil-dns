import argparse
import requests
import time
import hmac
import hashlib
import os

def parse_args():
    parser = argparse.ArgumentParser()
    
    parser.add_argument('--endpoint', '-e', help='request endpoint', type=int, required=True)
    
    parser.add_argument('--address', '-a', help='ip to find it\'s site', type=int, required=False)
    parser.add_argument('--site', '-s', help='site to find it\'s ip', type=int, required=False)
    
    parser.add_argument('--owner', '-o', help='owner of the message', type=int, required=True)
    parser.add_argument('--key-name', '-k', help='name of the key for hash', dest='key_name', type=str, required=True)
    parser.add_argument('--timeout', '-t', help='request timeout', type=int, required=True)
    parser.add_argument('--number-of-zeros', '-n', help='required number of zeros in hash', dest='n', default=4, type=int, required=False)

    return parser.parse_args()

def main():
    args = parse_args()
    
    request_params = {
        "owner": args.owner,
    }
    
    if args.address is not None:
        request_params["address"] = args.address
    else if args.site is not None:
        request_params["site"] = args.site
    else:
        assert False
    
    nonce = 1
    key = os.environ['SECRET_KEY']
    while True:
        request_params["nonce"] = nonce
        hash_key = hmac.new(key, json.dumps(dict(sorted(request_params.items()))).encode('utf-8'), hashlib.sha256).hexdigest()
        if hash_key[0:args.n] == "0" * args.n:
            break
    
    current_timestamp = int(time.time())
    request_params["timestamp"] = current_timestamp
    request_params["signature"] = hash_key
    
    request = requests.post(args.endpoint, json=request_params, timeout=args.timeout)
    print(f"Status: {request.status_code}.")
    print(f"Response: {request.json()}.")
    

if __name__ == '__main__':
    main()
