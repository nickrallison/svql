use crate::query_test;
use svql_query::prelude::*;

query_test!(
    name: test_sdffe_primitive,
    query: Sdffe<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/sdffe.il", "sdffe"),
    expect: 1
);

query_test!(
    name: test_adffe_primitive,
    query: Adffe<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/adffe.il", "adffe"),
    expect: 1
);

query_test!(
    name: test_sdff_primitive,
    query: Sdff<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/sdff.il", "sdff"),
    expect: 1
);

query_test!(
    name: test_sdffe_negative_on_sdff,
    query: Sdffe<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/sdff.il", "sdff"),
    expect: 0
);

query_test!(
    name: test_adff_primitive,
    query: Adff<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/adff.il", "adff"),
    expect: 1
);

query_test!(
    name: test_dffe_primitive,
    query: Dffe<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/dffe.il", "dffe"),
    expect: 1
);

query_test!(
    name: test_dff_any_primitive,
    query: DffAny<Search>,
    haystack: ("examples/patterns/basic/ff/rtlil/dff.il", "dff"),
    expect: 1
);
