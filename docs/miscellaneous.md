# Miscellaneous

This document covers a variety of topics that don't fit into any other category.

## Command Line Interface

Although the actual slicer system from mslicer is intended to be used through the interface in the `mslicer` binary, you can also use it as a standalone command-line application.
This allows for automating the slicing of models in bulk or as part of a more complicated build pipeline.
You can download the `slicer` binary from the [latest GitHub release](https://github.com/connorslade/mslicer/releases/latest).

Multiple meshes can be added by using the `--mesh` argument more than once.
If you want to change any properties of the mesh, like position, rotation, or scale, you can use the flag followed by a 3D vector (`x,y,z`).
These flags will modify the mesh defined most recently.
See the example below.

```bash
$ slicer --mesh teapot.stl --position 0,0,-0.05 --scale 2,2,2 --mesh frog.stl --position 100,0,0 output.goo
```

## Remote Print HTTP Status Proxy

Part of the process to upload a model to a printer with remote print is to serve the .goo file on an HTTP server, then send the download link to the printer over MQTT. Because remote print already has to run an HTTP server, this option exposes an API at `0.0.0.0:<http_port>/status`. Each time remote print starts, all server ports are randomized and printed to the log (check the console or the Log panel).

### Scriptable Widget

Before I explain how the API works, here is a cool thing you can use it for (if you have an iPhone). There is an app called [Scriptable](https://scriptable.app) that lets you write scripts for your phone in JavaScript. I have already written one that shows a widget with the status of the current print job. It needs to access the API server, so if you want to access it outside your home network, you will need to use some proxy service like ngrok.

Anyway, you can download the code for the widget from GitHub [here](https://gist.github.com/connorslade/5ee51da075fb0d7295f7a85cc774a5e0). Just make a new script in the app, paste the code in, and then you can make a Scriptable widget, edit it, and choose the script you made.

### The API

The status route returns a JSON array of printers, each with the following format.

```
struct Printer {
  machineId: String,
  attributes: Attributes,
  status: Status,
  lastUpdate: i64,
}
```

The `Attributes` structure is passed directly from the printer's initial handshake message. I'm honestly not sure what all the fields are for.

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
