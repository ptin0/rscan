import serial

ser = serial.Serial('/dev/pts/6', 115200)

print("AAA")

print(ser.name)

while True:
    s = ser.read()
    #if s == b'\xa0':
    #    s = b'\x11'
    ser.write(s)

