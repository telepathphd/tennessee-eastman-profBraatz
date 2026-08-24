# te_client

Thin Python client for TE identification experiments. Excitation (step / hold-time GBN) follows **mimo-sim** semantics; results export as `time,MV*,CV*` for `procest` in the sibling repo.

## Requirements

- Running `te-console` or `te-experiment` (Rust)
- Python 3.10+

## Example

```bash
cd rust && cargo run --release --bin te-console
```

```bash
python -m te_client.examples.run_gbn_reactor --output ./out
```

Feed `./out/te_gbn_reactor.csv` to `procest` in `SysID_Demo/mimo-sim/python/procest`.
