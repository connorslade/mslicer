# Miscellaneous

This document covers a variety of topics that don't fit into any other category.

## Remote Print HTTP Status Proxy

Part of the process to upload a model to a printer with remote print is to serve the .goo file on a http server, then send the download link to the printer over MQTT. Because remote print already has to run a HTTP server, this option exposes an api at `0.0.0.0:<http_port>/status`. Each time remote print starts all server port are randomized and printed to the log (check the console or the Log panel).

The status route returns an array of printers, each with the following format.

```
struct Printer {
  machineId: String,
  attributes: Attributes,
  status: Status,
  lastUpdate: i64,
}
```

The `Attributes` structure is passed directly from the printer's innitial handshake message. I'm honestly not sure what all the fields are for.

```
struct Attributes {
  Name: String,
  MachineName: String,
  ProtocolVersion: String,
  FirmwareVersion: String,
  Resolution: Resolution,
  MainboardIP: String,
  MainboardID: String,
  SDCPStatus: u8,
  LocalSDCPAddress: String,
  SDCPAddress: String,
  Capabilities: Capability[],
}

enum Capability {
  FILE_TRANSFER,
  PRINT_CONTROL
}
```

Finally, this data is sent from the printer over MQTT every few seconds.

```
struct Status {
  CurrentStatus: CurrentStatus,
  PreviousStatus: u8,
  PrintInfo: PrintInfo,
  FileTransferInfo: FileTransferInfo,
}

enum CurrentStatus {
  Ready,
  Busy,
  TransferringFile
}

enum PrintInfoStatus {
  None,
  InitialLower,
  Lowering,
  Exposure,
  Retracting,
  FinalRetract,
  Complete
}

enum FileTransferStatus {
  None,
  Done,
  Error
}
```
