from ecdsa import VerifyingKey
from ecdsa.util import sigdecode_der
from hashlib import sha256


if __name__ == '__main__':
    message = input('Enter message: ').encode()
    signature = bytes.fromhex(input('Enter signature: '))

    with open('public.pem', 'rb') as f:
        vk = VerifyingKey.from_pem(f.read())
    vk.verify(signature, message, hashfunc=sha256, sigdecode=sigdecode_der)
    print('Signature is successfully verified!')
