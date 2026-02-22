#include <cstdarg>
#include <cstddef>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>


/// Enum type encoding error codes of ERR packets.
///
/// UNKNOWN - somemthing gone very bad. The sender cannot sedcribe what exactly.
/// BUSY - the deivce is currently executing other task preventing execution of the command.
/// BROKEN - the received packet is broken, plese retransmit.
///
enum class ErrCodes {
  UNKNOWN,
  BUSY,
  BROKEN,
};

/// Enum type encoding packet types.
///
/// OK - acknowledgement that the command/mewasure has been received correctly
/// ERR - acknowledgement that the command/mewasure has been received incorrectly or the state of the device prevents the execution of a command
/// MOV - move motors by given offsets
/// MES - measurement data
/// ABORT - abort the current scan
/// PROG - contains scan parameters
/// FIN - scan has been finished
/// UNKNOWN - packet type is not known, something gone wrong. DO NOT SEND THIS VALUE!!!
///
enum class PacketType {
  OK = 0,
  ERR = 1,
  MOV = 2,
  MES = 3,
  ABORT = 4,
  PROG = 5,
  FIN = 6,
  UNKNOWN = 255,
};

/// Struct type describing the structure that envelps the packet.
/// Enveloped packet is ready to be framed and sent.
struct Envelope;


extern "C" {

void aaa(PacketType a, ErrCodes b, Envelope c);

} // extern "C"
