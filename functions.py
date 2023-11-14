import socket
import time
   
def connectToPixel(HOST, PORT):
    Pixel_conn = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    Pixel_conn.connect((HOST, PORT))

    #Might need to be changed to accept depending on how the pixelcom works
    return Pixel_conn

def disconnectFromPixel(Pixel_conn):
    Pixel_conn.close()
            
def sendToPixelTime(conn, time, connected):
    #print("Sending to pixel: ")
    #print(time)
    #print("\n")
    #print("Sending to pixel")
    #Build the message
    minutes = time // 60
    seconds = time % 60
    checksum = 1+14 + minutes + seconds 
    hex_string = "6908086900010e" + f"{minutes:02X}" + "00" + f"{seconds:02X}" + "0000" + hex(checksum)[2:] + "16"
    #print(hex_string)
    if connected:
        try:
            conn.sendall(bytes(hex_string, 'utf-8'))
            return ""
        except:
            return "Failed to send to pixel"
    return ""



def sendToPixelFlagCommand(conn, flag, connected):
    if flag == "no":
        hex_string = "6908086900010f00000000001016" #Turn of penalty board
    if flag == "yellow":
        hex_string = "6908086900011c00000000001d16" #Yellow flag
    if flag == "finish":
        hex_string = "6908086900010600000000646b16" #Finish flag
    if flag == "green":
        hex_string = "6908086900010600000000646b16" #Green flag #Needs to be modified with the correct hex value
    if flag == "red":
        hex_string = "6908086900010f00000000001016" #Red flag #Needs to be modified with the correct hex value
    if flag == "showTime":
        hex_string = "6908086900010f00000000001016" #Show time #Needs to be modified with the correct hex value
    
    if connected:
        try:
            conn.sendall(bytes(hex_string, 'utf-8'))
            return ""
        except:
            return "Failed to send to pixel"
    return ""

    #test



                