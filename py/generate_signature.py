from ecdsa import SigningKey, NIST256p


if __name__ == '__main__':
    sk = SigningKey.generate(curve=NIST256p)
    vk = sk.verifying_key
    with open('private.pem', 'wb') as f:
        f.write(sk.to_pem())
    with open('public.pem', 'wb') as f:
        f.write(vk.to_pem())
