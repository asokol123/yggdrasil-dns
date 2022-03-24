import argparse
import hashlib
import json
import os
import requests
import time


def parse_args():
    parser = argparse.ArgumentParser()
    
    parser.add_argument('--endpoint', '-e', help='request endpoint', type=int, required=True)
    parser.add_argument('--timeout', '-t', help='request timeout', type=int, default=5)
    parser.add_argument('--pow-zeros', '-z', help='required number of zeros in pow', type=int, default=4)
    subparsers = parser.add_subparsers(dest='command')

    register = subparsers.add_parser('register', help='sign user up')
    register.add_argument('--name', '-u', help='user to register', required=True)

    set_site = subparsers.add_parser('set_site', help='register a new site or update an existing one')
    set_site.add_argument('--site', '-s', help='site to register or update', required=True)
    set_site.add_argument('--address', '-a', help='site\'s ip', required=True)
    set_site.add_argument('--expires', '-E', help='site\'s expiration timestamp', type=int, required=True)
    set_site.add_argument('--owner', '-o', help='site\'s owner', required=True)

    get_site = subparsers.add_parser('get_site', help='get site by it\'s name')
    get_site.add_argument('--site', '-s', help='site to find', required=True)

    return parser.parse_args()


def main():
    args = parse_args()
    request_params = vars(args)
    endpoint = request_params.pop('endpoint')
    timeout = request_params.pop('timeout')
    pow_zeros = request_params.pop('pow_zeros')

    request_type = 'POST'
    if args.command == 'register':
        request_params['pubkey'] = os.environ['SECRET_KEY']
    elif args.command == 'set_site':
        request_params['signature'] = 'lol'  # TODO: signature
    elif args.command == 'set_site':
        request_type = 'GET'
    else:
        raise RuntimeError('Unknown command')

    request_params['timestamp'] = int(time.time())
    request_params['nonce'] = 1
    hash_key = hashlib.sha256(json.dumps(request_params).encode()).hexdigest()
    while not hash_key.startswith('0' * pow_zeros):
        request_params['nonce'] += 1
        hash_key = hashlib.sha256(json.dumps(request_params).encode()).hexdigest()
    
    request = requests.method(request_type, url=f'{endpoint}/{args.command}', json=request_params, timeout=timeout)
    print(f'Status: {request.status_code}')
    print(f'Response: {request.json()}')
    

if __name__ == '__main__':
    main()
