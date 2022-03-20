import argparse
import requests
import time
import hmac
import hashlib
import os

def parse_args():
    parser = argparse.ArgumentParser()
    
    parser.add_argument('--site', '-s', help='site to go', type=int, required=True)
    parser.add_argument('--address', '-a', help='???', type=int, required=True)
    parser.add_argument('--owner', '-o', help='owner of the message', type=int, required=True)
    parser.add_argument('--key-name', '-k', help='name of the key for hash', type=str, required=True)
    parser.add_argument('--signature', '-s', help='unique owner signature', type=str, required=True)
    parser.add_argument('--timeout', '-t', help='request timeout', type=int, required=True)

    return parser.parse_args()

def main():
    args = parse_args()
    
    request_params = {
        "owner": args.owner,
        "expire": args.expire,
    }
    
    key = os.environ.get(args.key)
    hash_key = hmac.new(key, request_params, hashlib.sha256).hexdigest()
    print(hash_key)
    
    current_timestamp = time.time()
    request_params["timestamp"] = current_timestamp
    request_params["key"] = hash_key
    
    request = requests.post(args.site, json=request_params, timeout=args.timeout)
    print(f"Status: {request.status_code}.")
    print(f"Request: {request.json()}.")
    

if __name__ == '__main__':
    main()
