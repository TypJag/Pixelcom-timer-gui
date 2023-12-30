from flask import Flask, render_template
from flask_socketio import SocketIO, emit
import time
import threading
import socket
import os
from functions import sendToPixelTime
from functions import connectToPixel
from functions import disconnectFromPixel
from functions import sendToPixelFlagCommand



app = Flask(__name__)
app.config['SECRET_KEY'] = 'your_secret_key'
socketio = SocketIO(app)

defaultTime = 600
timeLeft = defaultTime
halt = False
errorMsg = ""

showTime = False

HOST = "127.0.0.1" #Ip of test server
#HOST = "192.168.0.85" #Ip of pixelcom router
PORT = 24 # Port used by pixelcom

Pixel_conn = 0

@app.route('/')
def index():
  return render_template('index.html')

@socketio.on('connect')
def on_connect():
  print('Client connected')
  # Send a message to the client upon successful connection
  emit('message', {'data': 'Connected to server'})

@socketio.on('change')
def on_change(value):
  global timeLeft
  timeLeft = max(0, timeLeft + value)

  ping_clients()

@socketio.on('defaultChange')
def on_defaultChange(value):
  global defaultTime
  defaultTime = max(0, defaultTime + value)
  ping_clients()

@socketio.on('end')
def on_end():
  global timeLeft
  global isFinished
  #isFinished = True
  timeLeft = 0

  ping_clients()

@socketio.on('halt')
def on_halt():
  global halt
  halt = True

  ping_clients()

@socketio.on('unHalt')
def on_unHalt():
  global halt
  halt = False

  ping_clients()

@socketio.on('pixConnect')
def connect():
  global connected
  global Pixel_conn
  global errorMsg
  #Do some stuff to connect to pixelcom
  if connected == False:
    try:
      Pixel_conn = connectToPixel(HOST, PORT)
      connected = True
      errorMsg = ""
      ping_clients()
    except:
      errorMsg = "Error connecting to pixelcom"
      connected = False
      ping_clients()
      return

@socketio.on('pixDisconnect')
def disconnect():
  global connected
  global errorMsg
  #Do some stuff to disconnect to pixelcom
  if connected == True:
    try:
      disconnectFromPixel(Pixel_conn)
      connected = False
      ping_clients()
    except:
      errorMsg = "Error disconnecting from pixelcom"
      connected = False
      ping_clients()
      return 
    

@socketio.on('reset')
def on_reset():
  global timeLeft
  global isFinished
  timeLeft = defaultTime
  isFinished = False

  ping_clients()

@socketio.on('exit')
def on_exit():
  os._exit(0)

@socketio.on('flag')
def on_flag(flag):
  global Pixel_conn
  global connected
  global errorMsg
  global showTime

  if flag == "showTime":
    showTime = True
  else:
    showTime = False
    errorMsg = sendToPixelFlagCommand(Pixel_conn, flag, connected)
  
  ping_clients()


def ping_clients():
  global timeLeft
  global isFinished
  global connected
  global errorMsg
  global showTime

  localErrorMsg = ""

  # send to pixelcom displays
  if showTime:
    localErrorMsg = sendToPixelTime(Pixel_conn, timeLeft, connected)
  
  data = {
    'connected': connected,
    'timeLeft': format_time(timeLeft),
    'defaultTime': format_time(defaultTime),
    'errorMsg': errorMsg + " " + localErrorMsg,
  }
  # JSON data
  # Ping connected clients every second
  socketio.emit('ping', data, namespace='/')


def ping_loop():
  global timeLeft
  global halt
  global isFinished
  global connected
  isFinished = False
  connected = False
  while True:
    if halt == True:
      #print('Halted')
      time.sleep(1)
      #isFinished = False
      #Send the same time to pixelcom
      ping_clients()
    if halt == False and timeLeft > 0:
      #print('not halted')
      ping_clients()
      time.sleep(1)
      isFinished = False

      #Send updated time to pixelcom
      timeLeft = max(0, timeLeft - 1)
    if timeLeft == 0:
      #print('Time is up!')
      on_finish()


# External functions
def unHalt():
  global halt
  halt = False

def resetTime():
  global isFinished
  isFinished = False
  global timeLeft
  timeLeft = defaultTime
  
def on_finish():
  global isFinished
  if isFinished:
    return
  else:
    on_flag("finish")
    isFinished = True

def format_time(seconds):
    minutes = seconds // 60
    seconds = seconds % 60
    time_formatted = f"{minutes:02d}:{seconds:02d}"
    return time_formatted

def tcp_loop():
  global HOST
  global recvPORT
  global sendPORT
  global remainingLaps
  global Pixel_conn
  global Pixel_socket

  #Pixel_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
  #Pixel_socket.bind((sendHOST, sendPORT))
  #Pixel_socket.listen(1)

  #Pixel_conn, addr = Pixel_socket.accept()
  print("Connected to Pixel")

  

if __name__ == '__main__':
  threading.Thread(target=ping_loop).start()
  #threading.Thread(target=tcp_loop).start()
  socketio.run(app, host='127.0.0.1', port=5800, allow_unsafe_werkzeug=True)