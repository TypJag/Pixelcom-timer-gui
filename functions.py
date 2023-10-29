import socket
import time
   
def connectToPixel(sendHOST, sendPORT):
    Pixel_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    Pixel_socket.bind((sendHOST, sendPORT))
    Pixel_socket.listen(1)
    #Might need to be changed to accept depending on how the pixelcom works
    Pixel_conn, addr = Pixel_socket.accept()
    return Pixel_conn

def disconnectFromPixel(Pixel_conn):
    Pixel_conn.close()
            
def sendToPixelTime(conn, time):
    print("Sending to pixel: ")
    #print(time)
    #print("\n")
    #print("Sending to pixel")
    #Build the message
    minutes = time // 60
    seconds = time % 60
    checksum = 1+14 + minutes + seconds 
    hex_string = "6908086900010e" + f"{minutes:02X}" + "00" + f"{seconds:02X}" + "0000" + hex(checksum)[2:] + "16"
    print(hex_string)

    #conn.sendall(bytes(hex_string, 'utf-8'))  

def sendToPixelYellowFlag(conn):
    print("Sending Yellow Flag to pixel")

    hex_string = "6908086900011c00000000001d16"
    conn.sendall(bytes(hex_string, 'utf-8'))

def sendToPixelGreenFlag(conn):
    print("Sending Finish Flag to pixel")

    hex_string = "6908086900010600000000646b16"
    conn.sendall(bytes(hex_string, 'utf-8'))

def sendToPixelRedFlag(conn):
    print("Sending Red Flag to pixel")
    #Needs to be modified with the correct hex value
    hex_string = "6908086900010f00000000001016"
    conn.sendall(bytes(hex_string, 'utf-8'))

def sendToPixelShowTimeOrLaps(conn):
    print("Sending Show Time or Laps to pixel")
    #Needs to be modified with the correct hex value
    hex_string = "6908086900010f00000000001016"
    conn.sendall(bytes(hex_string, 'utf-8'))

def sendToPixelTurnOfPenaltyBoard(conn):
    print("Sending Turn Off Penalty Board to pixel")
    #Needs to be modified with the correct hex value
    hex_string = "6908086900010f00000000001016"
    conn.sendall(bytes(hex_string, 'utf-8'))

                