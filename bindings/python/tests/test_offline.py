
# Copyright 2023 IOTA Stiftung
# SPDX-License-Identifier: Apache-2.0

import json
import unittest
from iota_sdk import Block, Client, MnemonicSecretManager, Utils, SecretManager, OutputId, hex_to_utf8, utf8_to_hex, Bip44, CoinType, Irc27Metadata, Irc30Metadata


# Read the test vector
tv = {}
with open('../../sdk/tests/client/fixtures/test_vectors.json', "r", encoding="utf-8") as json_file:
    tv = json.load(json_file)

client = Client()


def test_mnemonic_address_generation():
    mnemonic_address_test_cases = tv['general']['address_generations']

    for test in mnemonic_address_test_cases:
        secret_manager = SecretManager(MnemonicSecretManager(test['mnemonic']))

        generated_address = secret_manager.generate_ed25519_addresses(
            coin_type=test['coin_type'],
            account_index=test['account_index'],
            bech32_hrp=test['bech32_hrp'],
            internal=test['internal'],
            start=test['address_index'],
            end=test['address_index'] + 1
        )

        assert test['bech32_address'] == generated_address[0]


def test_sign_verify_ed25519():
    secret_manager = MnemonicSecretManager(
        "acoustic trophy damage hint search taste love bicycle foster cradle brown govern endless depend situate athlete pudding blame question genius transfer van random vast")
    message = utf8_to_hex('IOTA')

    secret_manager = SecretManager(secret_manager)
    signature = secret_manager.sign_ed25519(
        message,
        # IOTA coin type
        Bip44(CoinType.IOTA),
    )
    assert signature.signature == '0x72bf2bc8fbc5dc56d657d7de8afa5208be1db025851e81031c754b371c7a29ce9f352d12df8207f9163316f81f59eb7725e5c0e4f3228e71ffe3764a9de6b10e'

    valid_signature = Utils.verify_ed25519_signature(
        signature,
        message,
    )
    assert valid_signature


class TestTypes(unittest.TestCase):
    def test_output_id(self):
        transaction_id = '0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649'
        output_index = 42
        output_id = OutputId(transaction_id, output_index)
        assert repr(output_id) == '0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00'

        new_output_id = OutputId.from_string(
            '0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00')
        assert repr(new_output_id) == '0x52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00'
        assert new_output_id.transaction_id == transaction_id
        assert new_output_id.output_index == output_index

        transaction_id_missing_0x_prefix = '52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649'
        with self.assertRaises(ValueError):
            OutputId(transaction_id_missing_0x_prefix, output_index)
        transaction_id__invalid_hex_prefix = '0052fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649'
        with self.assertRaises(ValueError):
            OutputId(transaction_id__invalid_hex_prefix, output_index)
        transaction_id_invalid_hex_char = '0xz2fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c649'
        with self.assertRaises(ValueError):
            OutputId(transaction_id_invalid_hex_char, output_index)
        invalid_output_index = 129
        with self.assertRaises(ValueError):
            OutputId(transaction_id, invalid_output_index)
        output_id_missing_0x_prefix = '52fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00'
        with self.assertRaises(ValueError):
            OutputId.from_string(output_id_missing_0x_prefix)
        output_id_invalid_hex_char = '0xz2fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00'
        with self.assertRaises(ValueError):
            OutputId.from_string(output_id_invalid_hex_char)
        output_id_invalid_hex_prefix = '0052fdfc072182654f163f5f0f9a621d729566c74d10037c4d7bbb0407d1e2c6492a00'
        with self.assertRaises(ValueError):
            OutputId.from_string(output_id_invalid_hex_prefix)


def test_hex_utf8():
    utf8_data = "Don't panic!"
    hex_data = '0x446f6e27742070616e696321'
    assert utf8_to_hex(utf8_data) == hex_data
    assert hex_to_utf8(hex_data) == utf8_data


def test_block():
    block_dict = {"protocolVersion": 2, "parents": ["0x28dbf8f5005c0de388cbbf23d14645a579fc0cb8278ad9cdc5a4252c7e8f0ed3", "0x440dbc33bf05c334c6d49f06514526d7f3e3c758028a2e87636e19f886290900",
                                                    "0xd76cdb7acf228ecdad590a42b91acc077c1518c1a271411229e33e050fc19b44", "0xecef38d3af7e63da78a5e70128efe371f2191088b31879f7b0e81da657fa21c6"], "payload": {"type": 5, "tag": "0x68656c6c6f", "data": "0x68656c6c6f"}, "nonce": "6917529027641139843"}
    block = Block.from_dict(block_dict)
    assert block.id() == "0x7ce5ad074d4162e57f83cfa01cd2303ef5356567027ce0bcee0c9f57bc11656e"


def test_irc_27():
    metadata = Irc27Metadata(
        "video/mp4",
        "https://ipfs.io/ipfs/QmPoYcVm9fx47YXNTkhpMEYSxCD3Bqh7PJYr7eo5YjLgiT",
        "Shimmer OG NFT",
        description="The original Shimmer NFT"
    )
    metadata_dict = {
        "standard": "IRC27",
        "version": metadata.version,
        "type": metadata.type,
        "uri": metadata.uri,
        "name": metadata.name,
        "collectionName": metadata.collectionName,
        "royalties": metadata.royalties,
        "issuerName": metadata.issuerName,
        "description": metadata.description,
        "attributes": metadata.attributes
    }
    metadata_deser = Irc27Metadata.from_dict(metadata_dict)
    assert metadata == metadata_deser


def test_irc_30():
    metadata = Irc30Metadata(
        "FooCoin",
        "FOO",
        3,
        description="FooCoin is the utility and governance token of FooLand, \
                a revolutionary protocol in the play-to-earn crypto gaming field.",
        url="https://foocoin.io/",
        logoUrl="https://ipfs.io/ipfs/QmR36VFfo1hH2RAwVs4zVJ5btkopGip5cW7ydY4jUQBrkR"
    )
    metadata_dict = {
        "standard": "IRC30",
        "name": metadata.name,
        "description": metadata.description,
        "decimals": metadata.decimals,
        "symbol": metadata.symbol,
        "url": metadata.url,
        "logoUrl": metadata.logoUrl
    }
    metadata_deser = Irc30Metadata.from_dict(metadata_dict)
    assert metadata == metadata_deser
