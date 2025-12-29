
## Hummingbird V2

2025-12-28T23:35:26.286328Z  INFO example_query: Loading design...
2025-12-28T23:35:26.286339Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/fixtures/larger_designs/json/hummingbirdv2/e203_soc_netlist.json (e203_soc_top)
2025-12-28T23:35:35.179377Z  INFO example_query: Building context...
2025-12-28T23:35:35.179411Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/basic/and/verilog/and_gate.v (and_gate)
2025-12-28T23:35:35.194256Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/basic/or/verilog/or_gate.v (or_gate)
2025-12-28T23:35:35.207500Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/basic/not/verilog/not_gate.v (not_gate)
2025-12-28T23:35:35.213953Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/security/access_control/locked_reg/rtlil/async_en.il (async_en)
2025-12-28T23:35:35.219911Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/security/access_control/locked_reg/rtlil/sync_en.il (sync_en)
2025-12-28T23:35:35.226104Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/security/access_control/locked_reg/rtlil/async_mux.il (async_mux)
2025-12-28T23:35:35.232095Z  INFO svql_driver::driver: Loading design: /home/nick/Downloads/svql/examples/patterns/security/access_control/locked_reg/rtlil/sync_mux.il (sync_mux)
2025-12-28T23:35:35.238156Z  INFO example_query: Instantiating query...
2025-12-28T23:35:35.238171Z  INFO example_query: Executing query...
2025-12-28T23:35:35.238183Z  INFO svql_query::security::cwe1234::unlock_logic: UnlockLogic::query: starting CWE1234 unlock pattern search
2025-12-29T00:00:49.510859Z  INFO svql_query::composites::rec_or: RecOr::query: starting recursive OR gate search
2025-12-29T00:00:49.518004Z  INFO svql_query::composites::rec_or: RecOr::query: Found 2099 total OR gates in design
2025-12-29T00:00:49.776297Z  INFO svql_query::composites::rec_or: RecOr::query: Layer 2 has 2901 matches
2025-12-29T00:00:50.155505Z  INFO svql_query::composites::rec_or: RecOr::query: Layer 3 has 6848 matches
2025-12-29T00:00:50.339846Z  INFO svql_query::security::cwe1234::unlock_logic: UnlockLogic::query: Found 318101 AND gates, 11848 RecOR trees, 1226 NOT gates
2025-12-29T00:01:38.458811Z  INFO svql_query::security::cwe1234::unlock_logic: UnlockLogic::query: Found 48572 valid (RecOr, AND) pairs
2025-12-29T00:01:38.734785Z  INFO svql_query::security::cwe1234::unlock_logic: UnlockLogic::query: Found 1084 final valid patterns
2025-12-29T00:01:39.265361Z  INFO example_query: Found 0 matches for old query
18148.28user 292.01system 26:42.24elapsed 1150%CPU (0avgtext+0avgdata 3569004maxresident)k
63744inputs+768168outputs (155major+83513514minor)pagefaults 0swaps