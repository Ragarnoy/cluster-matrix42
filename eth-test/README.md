# eth-test - cluster-net Hardware Test

This project tests the `cluster-net` library on RP2350 with W6100 ethernet.

## Current Status

✅ **Working**: This project includes a compatibility layer (`src/compat.rs`) that bridges the version incompatibility between:
- `reqwless 0.13` (uses `embedded-nal-async 0.8`)
- `embassy-net` (uses `embedded-nal-async 0.9`)

The `StackAdapter` implements embedded-nal-async 0.8 traits for `embassy-net::Stack`, allowing `cluster-net` to work with embassy-net.

### Compatibility Layer Details

The adapter:
- Implements `TcpConnect` and `Dns` traits from embedded-nal-async 0.8
- Stores TCP socket buffers internally (4KB RX/TX)
- Handles one connection at a time (sufficient for reqwless's sequential request pattern)
- Uses `UnsafeCell` for buffer storage (safe in single-threaded embassy executor)

## Hardware Configuration

- **Chip**: RP2350
- **Ethernet**: WIZnet W6100
- **Pin Mapping**:
  - MISO = PIN_16
  - MOSI = PIN_19
  - SCLK = PIN_18
  - CSn = PIN_17
  - RSTn = PIN_20
  - INTn = PIN_21

## Expected Behavior

1. Initialize W6100 ethernet chip
2. Obtain IP address via DHCP
3. Test HTTP requests to fetch cluster data
4. (With TLS feature) Test HTTPS requests
5. Enter continuous polling mode

## Future Work

- [ ] Test with real cluster API server
- [ ] Add retry logic and error recovery
- [ ] Implement caching to reduce network requests
- [ ] Add status LED indicators
- [ ] Consider migrating to embedded-nal-async 0.9 when reqwless updates
