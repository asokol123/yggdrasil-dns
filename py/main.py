import argparse
import hashlib
import json
import os
import requests
import time

from ecdsa import SigningKey
from ecdsa.util import sigencode_der

DEFAULT_DIFFICULTY = 4
DEFAULT_TIMEOUT = 5


class TopLevelArgs(argparse.Action):
    def __call__(self, parser, namespace, values, option_string=None):
        if not getattr(namespace, 'options', None):
            namespace.options = {}
        namespace.options[self.dest] = values


def parse_args():
    parser = argparse.ArgumentParser()

    parser.add_argument('--endpoint', '-e', help='request endpoint', action=TopLevelArgs, default=argparse.SUPPRESS)
    parser.add_argument('--timeout', '-t', help='request timeout', type=int, action=TopLevelArgs,
                        default=argparse.SUPPRESS)
    parser.add_argument('--pow-zeros', '-z', help='required number of zeros in pow', type=int, action=TopLevelArgs,
                        default=argparse.SUPPRESS)
    subparsers = parser.add_subparsers(dest='command')

    register = subparsers.add_parser('register', help='sign user up')
    register.add_argument('--name', '-u', help='user to register', required=True)

    set_site = subparsers.add_parser('set_site', help='register a new site or update an existing one')
    set_site.add_argument('--site', '-s', help='site to register or update', required=True)
    set_site.add_argument('--address', '-a', help='site\'s ip', required=True)
    set_site.add_argument('--expires', '-E', help='site\'s expiration timestamp', type=int, required=True)
    set_site.add_argument('--owner', '-o', help='site\'s owner', required=True)
    set_site.add_argument('--signature-filename', '-f', help='file with a signature', required=True)

    get_site = subparsers.add_parser('get_site', help='get site by it\'s name')
    get_site.add_argument('--site', '-s', help='site to find', required=True)

    return vars(parser.parse_args())


def main():
    request_params = parse_args()
    options, command = request_params.pop('options'), request_params.pop('command')
    request_params['timestamp'] = int(time.time())

    request_type = 'POST'
    if command == 'register':
        request_params['pubkey'] = os.environ['PUBLIC_KEY']
    elif command == 'set_site':
        filename = request_params.pop('signature_filename')
        with open(filename, 'r') as f:
            sk = SigningKey.from_pem(f.read())

        message = request_params['owner'] + request_params['site'] + str(request_params['timestamp'])
        request_params['signature'] = sk.sign(message.encode(), hashfunc=hashlib.sha256, sigencode=sigencode_der).hex()
    elif command == 'get_site':
        request_type = 'GET'
    else:
        raise RuntimeError('Unknown command')

    request_params['nonce'] = 1
    hash_key = hashlib.sha256(json.dumps(request_params).encode()).hexdigest()
    while not hash_key.startswith('0' * options.get('pow_zeros', DEFAULT_DIFFICULTY)):
        request_params['nonce'] += 1
        hash_key = hashlib.sha256(json.dumps(request_params).encode()).hexdigest()

    request = requests.request(request_type, url=f'http://{options["endpoint"]}/{command}', json=request_params,
                               timeout=options.get('timeout', DEFAULT_TIMEOUT))
    print(f'Status: {request.status_code}')
    print(f'Response: {request.json()}')


if __name__ == '__main__':
    main()
