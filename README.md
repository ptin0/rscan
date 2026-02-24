# Rscanner

This is my part of the scanner project created during masters programme.

# Objective
Use a lidar sensor and two step morots for radial scan of physical features.

## My Tasks
- design a communication protocol PC-scanner.
- Write a Rust based Linux client, that talks with the scanner.
- Write a Python script that visalizes data from the scanner in Blender.

## Other teammates tasks (not present here)
- Inegrate motors, motor drivers, lidar and MCU together.
- Write a firmware that controls the device and communicates with PC.

# Architecture
Insert DrawIO block diagram.

## Measurement
The measurement is conducted by changing the lidar rotation.
Lidar rotates in Z axis, gatehering points, which form a line.
After each line, X axis changes and a new line is being gathered.
The lines, form a mesh of points, that show the scanned geometry.

# Communication protocol

The device communicates via UART over USB OTG.
The communication is started by the PC client.
The client can issue `MOV` commands to move motors to setup sthe starting position of lidar.
Once the lidar orientation is correct, the client issues `PROG` command, that contains parameters of the scan.

Each message requires acknowledgement, except `OK` and `ERR` messages.
Devices should consider packet as lost, unless `OK` or `ERR` with the same `message ID` has been received and retransmit the packet.
`Message ID` also enables discarding the duplicates.

The packets are sent using [COBS](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing) frames.
This ensures that start and end of packet byte cannot accidently occur anywhere else than start byte and end delimiter byte.

## Header
Each packet starts with a 5 byte header

| Field       | Description                                      |
| ----------- | -----------                                      |
| LEN         | Length of the packet (including header) in bytes |
| MSG         | Message type code                                |
| ID          | Message ID                                       |

`Message types` are described later.
`Message ID` must not repeat for at least two consecutive packets. Recommended approach is to use overflowing incrementation.

## MOV

Message issuing move command to set the starting lidar orientation.

| Field       | Description                                      |
| ----------- | -----------                                      |
| HEADER      | Standard header                                  |
| AXIS        | Axis of the rotation (Horizon, Azimuth)          |
| SIDE        | Clockwise or counter-clockwise rotation          |
| STEPS       | Step count                                       |

## PROG

Message issuing start of measurements command. It contains measurement parameters

| Field        | Description                                                       |
| -----------  | -----------                                                       |
| HEADER       | Standard header                                                   |
| Z STEP COUNT | How many points the scan has                                      |
| X STEP SIZE  | How many lines a line in the scan has                             |

## MES

A single measurement point data.

| Field        | Description                                                       |
| -----------  | -----------                                                       |
| HEADER       | Standard header                                                   |
| MES          | unsigned 32 bit distance value                                    |

Since the communication is sequential, and both devices know the scan parameters, there is no need for more data to be passed each point.

## ABORT

Aborts the scan.

| Field        | Description                                                       |
| -----------  | -----------                                                       |
| HEADER       | Standard header                                                   |

## FIN

Confirms that the scan has ended.

| Field        | Description                                                       |
| -----------  | -----------                                                       |
| HEADER       | Standard header                                                   |

## OK

Acknowledges a message.

| Field        | Description                                                       |
| -----------  | -----------                                                       |
| HEADER       | Standard header                                                   |
| MSG ID       | The ID of a message that is being acknowledged                    |

## ERR

Acknowledges the messages, but informs about an error

| Field        | Description                                                       |
| -----------  | -----------                                                       |
| HEADER       | Standard header                                                   |
| MSG ID       | The ID of a message that is being acknowledged                    |
| ERR TYPE     | Error code                                                        |

Currently supported errors are:
- UNKNOWN - Unexpected error
- BUSY - The scanner is busy with a scan and can't perform the operation.
- BROKEN - The packet is broken.

