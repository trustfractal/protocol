//TODO(melatron): Remove js-test folder
import pkg from '../pkg/sier_codec_js.js';

const JSON_STRUCT_DEF = `
struct Corge {
    gz :u64;
    op :bool;
}

struct Foo {
    bar :u64;
    baz :string;
    qux :List<u64>;
    corge :Corge;
}`;

const FOO =
{
    "bar": 42,
    "baz": "abc",
    "qux": [4, 2],
    "corge": {
        "gz": 42,
        "op": true
    }
};
console.log(FOO);

let sier = pkg.serialize(FOO, JSON_STRUCT_DEF, "Foo");
console.log(sier);
let json = pkg.deserialize(sier, JSON_STRUCT_DEF);
console.log(json);
