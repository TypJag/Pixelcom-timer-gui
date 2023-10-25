import socket
import time


def checkHasLeaderPassedAndLaps(host_socket, currentlap):
    
    while True:
        try:
            data = host_socket.recv(1024)
        except:
            print("Failed to read data! ")

        sdata = data.decode('latin-1')
        sdata = sdata.split(",")
        if sdata[0] == '$F':
            if sdata[1] != currentlap:
                if sdata[1] == '9999':
                    if currentlap != 1:
                        return True, 1
                else:
                    return True, sdata[1]
            else:
                return False, currentlap
        elif sdata[0] == '':
            return False, currentlap    

            
def sendToPixel(conn, remainingLaps):
    print("Sending to pixel: ")
    tempremLaps = int(remainingLaps) -1
    print(tempremLaps)
    print("\n")
    #print("Sending to pixel")
    for i in range(5):
        tempremLaps = int(remainingLaps) -1
        if tempremLaps != -1:            
                conn.sendall(bytes(f'$F,{str(tempremLaps)},"00:00:00","00:00:00","00:00:00","      "\r\n', encoding='utf8'))



def sendToPixel2(conn, remainingLaps):
    print("Sending to pixel2: ")
    print(remainingLaps)
    #print("\n")
    for i in range(5):
        conn.sendall(bytes(f'$F,{str(remainingLaps)},"00:00:00","00:00:00","00:00:00","      "\r\n', encoding='utf8'))

    
def connectToClient(conn,socket):
    from server import sendHOST,sendPORT   
    #socket.close()

    conn, addr = socket.accept()
    print("Hi ",addr)

