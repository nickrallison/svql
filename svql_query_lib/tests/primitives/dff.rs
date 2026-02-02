use crate::query_test;
use svql_query_lib::primitives::dff::*;

query_test!(
    name: test_sdffe_primitive,
    query: Sdffe,
    haystack: ("examples/patterns/basic/ff/rtlil/sdffe.il", "sdffe"),
    expect: 1
);

query_test!(
    name: test_adffe_primitive,
    query: Adffe,
    haystack: ("examples/patterns/basic/ff/rtlil/adffe.il", "adffe"),
    expect: 1
);

query_test!(
    name: test_sdff_primitive,
    query: Sdff,
    haystack: ("examples/patterns/basic/ff/rtlil/sdff.il", "sdff"),
    expect: 1
);

query_test!(
    name: test_sdffe_negative_on_sdff,
    query: Sdffe,
    haystack: ("examples/patterns/basic/ff/rtlil/sdff.il", "sdff"),
    expect: 0
);

query_test!(
    name: test_adff_primitive,
    query: Adff,
    haystack: ("examples/patterns/basic/ff/rtlil/adff.il", "adff"),
    expect: 1
);

query_test!(
    name: test_dffe_primitive,
    query: Dffe,
    haystack: ("examples/patterns/basic/ff/rtlil/dffe.il", "dffe"),
    expect: 1
);

query_test!(
    name: test_dff_any_primitive,
    query: DffAny,
    haystack: ("examples/patterns/basic/ff/rtlil/dff.il", "dff"),
    expect: 1
);
