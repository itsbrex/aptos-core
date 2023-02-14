// Copyright © Aptos Labs
// SPDX-License-Identifier: Apache-2.0

/* eslint-disable max-len */

// ecosystem/typescript/sdk/src/move_scripts/token_transfer_with_opt_in
export const TOKEN_TRANSFER_OPT_IN =
  "a11ceb0b0500000006010004020408030c0a05161d073339086c400000010100020700010307000104030100010504020007060c0508000800030503010801000405080008000304060c0801050306737472696e6705746f6b656e06537472696e6707546f6b656e4964136372656174655f746f6b656e5f69645f726177087472616e73666572000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000030000010c0b010b020b030b0411000c070b000b070b050b06110102";

/*
Follow these steps to get the ABI strings:

Go to the package directory of the relevant Move module, e.g. if you're trying
to get the ABI for the `transfer` function of `aptos_account.move`, go to
the directory `aptos-move/framework/aptos-framework`.

Compile the Move packages with the Aptos CLI:
```
aptos move compile --included-artifacts all
```
This `--included-artifacts all` argument is necessary to generate ABIs.

Find the ABI files under the `build` directory and convert the ABI files to hex strings.
On Mac and Linux, this can be done with this command:
```
cat <ABI_FILE_PATH> | od -v -t x1 -A n | tr -d ' \n'
```
For example:
```
cat build/AptosFramework/abis/aptos_account/transfer.abi | od -v -t x1 -A n | tr -d ' \n'
```
*/
export const TOKEN_ABIS = [
  // aptos-token/build/AptosToken/abis/token/create_collection_script.abi
  "01186372656174655F636F6C6C656374696F6E5F736372697074000000000000000000000000000000000000000000000000000000000000000305746F6B656E3020637265617465206120656D70747920746F6B656E20636F6C6C656374696F6E207769746820706172616D65746572730005046E616D6507000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67000B6465736372697074696F6E07000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67000375726907000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E6700076D6178696D756D020E6D75746174655F73657474696E670600",
  // aptos-token/build/AptosToken/abis/token/create_token_script.abi
  "01136372656174655F746F6B656E5F736372697074000000000000000000000000000000000000000000000000000000000000000305746F6B656E1D2063726561746520746F6B656E20776974682072617720696E70757473000D0A636F6C6C656374696F6E07000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E6700046E616D6507000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67000B6465736372697074696F6E07000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67000762616C616E636502076D6178696D756D020375726907000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E670015726F79616C74795F70617965655F61646472657373041A726F79616C74795F706F696E74735F64656E6F6D696E61746F720218726F79616C74795F706F696E74735F6E756D657261746F72020E6D75746174655F73657474696E6706000D70726F70657274795F6B6579730607000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67000F70726F70657274795F76616C7565730606010E70726F70657274795F74797065730607000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E6700",
  // aptos-token/build/AptosToken/abis/token/direct_transfer_script.abi
  "01166469726563745f7472616e736665725f736372697074000000000000000000000000000000000000000000000000000000000000000305746f6b656e0000051063726561746f72735f61646472657373040a636f6c6c656374696f6e07000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700046e616d6507000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e67001070726f70657274795f76657273696f6e0206616d6f756e7402",
  // aptos-token/build/AptosToken/abis/token_transfers/offer_script.abi
  "010C6F666665725F73637269707400000000000000000000000000000000000000000000000000000000000000030F746F6B656E5F7472616E7366657273000006087265636569766572040763726561746F72040A636F6C6C656374696F6E07000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E6700046E616D6507000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67001070726F70657274795F76657273696F6E0206616D6F756E7402",
  // aptos-token/build/AptosToken/abis/token_transfers/claim_script.abi
  "010C636C61696D5F73637269707400000000000000000000000000000000000000000000000000000000000000030F746F6B656E5F7472616E73666572730000050673656E646572040763726561746F72040A636F6C6C656374696F6E07000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E6700046E616D6507000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67001070726F70657274795F76657273696F6E02",
  // aptos-token/build/AptosToken/abis/token_transfers/cancel_offer_script.abi
  "011363616E63656C5F6F666665725F73637269707400000000000000000000000000000000000000000000000000000000000000030F746F6B656E5F7472616E7366657273000005087265636569766572040763726561746F72040A636F6C6C656374696F6E07000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E6700046E616D6507000000000000000000000000000000000000000000000000000000000000000106737472696E6706537472696E67001070726F70657274795F76657273696F6E02",
  // aptos-token/build/AptosToken/abis/token/mutate_token_properties.abi
  "01176d75746174655f746f6b656e5f70726f70657274696573000000000000000000000000000000000000000000000000000000000000000305746f6b656eba02206d75746174652074686520746f6b656e2070726f706572747920616e64207361766520746865206e65772070726f706572747920696e20546f6b656e53746f72650a2069662074686520746f6b656e2070726f70657274795f76657273696f6e20697320302c2077652077696c6c206372656174652061206e65772070726f70657274795f76657273696f6e2070657220746f6b656e20746f2067656e65726174652061206e657720746f6b656e5f69642070657220746f6b656e0a2069662074686520746f6b656e2070726f70657274795f76657273696f6e206973206e6f7420302c2077652077696c6c206a75737420757064617465207468652070726f70657274794d617020616e642075736520746865206578697374696e6720746f6b656e5f6964202870726f70657274795f76657273696f6e2900090b746f6b656e5f6f776e6572040763726561746f72040f636f6c6c656374696f6e5f6e616d6507000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e67000a746f6b656e5f6e616d6507000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e670016746f6b656e5f70726f70657274795f76657273696f6e0206616d6f756e7402046b6579730607000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e67000676616c7565730606010574797065730607000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700",
  // aptos-token/build/AptosToken/abis/token/opt_in_direct_transfer.abi
  "01166f70745f696e5f6469726563745f7472616e73666572000000000000000000000000000000000000000000000000000000000000000305746f6b656e000001066f70745f696e00",
  // aptos-token/build/AptosToken/abis/token/burn.abi
  "01046275726e000000000000000000000000000000000000000000000000000000000000000305746f6b656e20204275726e206120746f6b656e2062792074686520746f6b656e206f776e657200051063726561746f72735f61646472657373040a636f6c6c656374696f6e07000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700046e616d6507000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e67001070726f70657274795f76657273696f6e0206616d6f756e7402",
  // aptos-token/build/AptosToken/abis/token/burn_by_creator.abi
  "010f6275726e5f62795f63726561746f72000000000000000000000000000000000000000000000000000000000000000305746f6b656e6a204275726e206120746f6b656e2062792063726561746f72207768656e2074686520746f6b656e2773204255524e41424c455f42595f43524541544f5220697320747275650a2054686520746f6b656e206973206f776e65642061742061646472657373206f776e65720005056f776e6572040a636f6c6c656374696f6e07000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e6700046e616d6507000000000000000000000000000000000000000000000000000000000000000106737472696e6706537472696e67001070726f70657274795f76657273696f6e0206616d6f756e7402",
];

export const COIN_ABIS = [
  // aptos-framework/build/AptosFramework/abis/coin/transfer.abi
  "01087472616E73666572000000000000000000000000000000000000000000000000000000000000000104636F696E3C205472616E73666572732060616D6F756E7460206F6620636F696E732060436F696E54797065602066726F6D206066726F6D6020746F2060746F602E0109636F696E5F747970650202746F0406616D6F756E7402",
  // aptos-framework/build/AptosFramework/abis/aptos_account/transfer.abi
  "01087472616e7366657200000000000000000000000000000000000000000000000000000000000000010d6170746f735f6163636f756e7400000202746f0406616d6f756e7402",
];
