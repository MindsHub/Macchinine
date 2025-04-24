#include <IRremote.h>
#include <Servo.h>

/*#define f 16736925  // FORWARD
#define b 16754775  // BACK
#define l 16720605  // LEFT
#define r 16761405  // RIGHT
#define s 16712445  // STOP*/
#define KEY1 16738455 //Line Teacking mode
#define KEY2 16750695 //Obstacles Avoidance mode
#define KEY3 16756815
#define KEY4 16724175
#define KEY5 16718055
#define KEY6 16743045
#define KEY7 16716015
#define KEY8 16726215
#define KEY9 16734885
#define KEY0 16730805
#define KEY_STAR 16728765
#define KEY_HASH 16732845

#define RECV_PIN  12
#define ECHO_PIN  A4
#define TRIG_PIN  A5
#define ENA 5
#define ENB 6
#define IN1 7
#define IN2 8
#define IN3 9
#define IN4 11
#define LED_Pin 13
#define LineTeacking_Pin_Right  10
#define LineTeacking_Pin_Middle 4
#define LineTeacking_Pin_Left   2
#define LineTeacking_Read_Right   !digitalRead(10)
#define LineTeacking_Read_Middle  !digitalRead(4)
#define LineTeacking_Read_Left    !digitalRead(2)
#define carSpeed 250

Servo servo;
IRrecv irrecv(RECV_PIN);
decode_results results;
unsigned long IR_PreMillis;
unsigned long LT_PreMillis;
int rightDistance = 0, leftDistance = 0, middleDistance = 0;

enum FUNCTIONMODE{
  IDLE,
  LineTeacking,
  ObstaclesAvoidance,
  Bluetooth,
  IRremote
} func_mode = IDLE;

enum MOTIONMODE {
  STOP,
  FORWARD,
  BACK,
  LEFT,
  RIGHT
} mov_mode = STOP;

unsigned char moveFromBluetooth=0;

void delays(unsigned long t) {
  for(unsigned long i = 0; i < t; i++) {
    getBTData();
    //getIRData();
    delay(1);
  }
}

int getDistance() {
  digitalWrite(TRIG_PIN, LOW);
  delayMicroseconds(2);
  digitalWrite(TRIG_PIN, HIGH);
  delayMicroseconds(10);
  digitalWrite(TRIG_PIN, LOW);
  return (int)pulseIn(ECHO_PIN, HIGH) / 58;
}

void set_motors(int sx, int dx){
  analogWrite(ENA, abs(sx));
  analogWrite(ENB, abs(dx));
  if(sx>0){
    digitalWrite(IN1,HIGH);
    digitalWrite(IN2,LOW);
  }else{
    digitalWrite(IN1,LOW);
    digitalWrite(IN2,HIGH);
  }
  if(dx>0){
    digitalWrite(IN3,LOW);
    digitalWrite(IN4,HIGH);
  }else{
    digitalWrite(IN3,HIGH);
    digitalWrite(IN4,LOW);
  }
}


signed int conv(signed char c) {
    if (c >> 3) {
        c |= 0xf0;
    }

    switch(c){
        case -7: return -240;//-255;
        case -6: return -219;
        case -5: return -182;
        case -4: return -146;
        case -3: return -109;
        case -2: return -73;
        case -1: return -36;
        case 0: default: return 0;
        case 1: return 36;
        case 2: return 73;
        case 3: return 109;
        case 4: return 146;
        case 5: return 182;
        case 6: return 219;
        case 7: return 240;//255;
    }
}

int last_sent=0;
unsigned long last_received=0;
void getBTData() {
  if(Serial.available()) {
    while(Serial.available())
      moveFromBluetooth=Serial.read();
    last_received=millis();
   
    /*switch(Serial.read()) {
      case 'f': func_mode = Bluetooth; mov_mode = FORWARD;  break;
      case 'b': func_mode = Bluetooth; mov_mode = BACK;     break;
      case 'l': func_mode = Bluetooth; mov_mode = LEFT;     break;
      case 'r': func_mode = Bluetooth; mov_mode = RIGHT;    break;
      case 's': func_mode = Bluetooth; mov_mode = STOP;     break;
      case '1': func_mode = LineTeacking;                   break;
      case '2': func_mode = ObstaclesAvoidance;             break;
      default:  break;
    } */
  } else if (millis()-last_received>5000) {
    moveFromBluetooth=0;
    last_received=millis();
  }
  /*if(millis()-last_sent>100){
      last_sent=millis();
      Serial.write((unsigned char) middleDistance);
    }*/
}/*
void getIRData() {
  if (irrecv.decode(&results)){
    IR_PreMillis = millis();
    switch(results.value){
      case f:   func_mode = IRremote; mov_mode = FORWARD;  break;
      case b:   func_mode = IRremote; mov_mode = BACK;     break;
      case l:   func_mode = IRremote; mov_mode = LEFT;     break;
      case r:   func_mode = IRremote; mov_mode = RIGHT;    break;
      case s:   func_mode = IRremote; mov_mode = STOP;     break;
      case KEY1:  func_mode = LineTeacking;                break;
      case KEY2:  func_mode = ObstaclesAvoidance;          break;
      default: break;
    }
    irrecv.resume();
  }
}*/

/*
void irremote_mode() {
  if(func_mode == IRremote){
    switch(mov_mode){
      case FORWARD: forward();  break;
      case BACK:    back();     break;
      case LEFT:    left();     break;
      case RIGHT:   right();    break;
      case STOP:    stop();     break;
      default: break;
    }
    if(millis() - IR_PreMillis > 500){
      mov_mode = STOP;
      IR_PreMillis = millis();
    }
  }
}*/

void line_teacking_mode() {
  if(LineTeacking_Read_Right || LineTeacking_Read_Middle || LineTeacking_Read_Left){
    int sx = conv((moveFromBluetooth&0xf0)>>4);
    int dx = conv(moveFromBluetooth&0x0f);
    if (sx+dx > 0) {
      set_motors(sx>dx ? -255 : -200, sx>dx ? -200 : -255);
    } else {
      set_motors(sx>dx ? 255 : 200, sx>dx ? 200 : 255);
    }
    while(LineTeacking_Read_Right || LineTeacking_Read_Middle || LineTeacking_Read_Left) getBTData();
    delays(100);
  }
}

void obstacles_avoidance_mode() {
    middleDistance = getDistance();

    if(middleDistance <= 20) {
      //stop();
      set_motors(0,0);
      delays(500);
      servo.write(10);
      delays(1000);
      rightDistance = getDistance();

      delays(500);
      servo.write(90);
      delays(1000);
      servo.write(170);
      delays(1000);
      leftDistance = getDistance();

      delays(500);
      servo.write(90);
      delays(1000);
      if(rightDistance > leftDistance) {
        set_motors(-255,255);
        //right();
        delays(360);
      } else if(rightDistance < leftDistance) {
        set_motors(255,-255);
        //left();
        delays(360);
      } else if((rightDistance <= 20) || (leftDistance <= 20)) {
        set_motors(-255,-255);
        //back();
        delays(180);
      } else {
        //set_motors(255,255);
        //forward();
      }
   }
}
void bluetooth_mode() {
  int sx = conv((moveFromBluetooth&0xf0)>>4);
  int dx = conv(moveFromBluetooth&0x0f);
  set_motors(sx, dx);
}


void setup() {
  Serial.begin(115200);
  servo.attach(3,500,2400);// 500: 0 degree  2400: 180 degree
  servo.write(90);
  irrecv.enableIRIn();
  pinMode(ECHO_PIN, INPUT);
  pinMode(TRIG_PIN, OUTPUT);
  pinMode(IN1, OUTPUT);
  pinMode(IN2, OUTPUT);
  pinMode(IN3, OUTPUT);
  pinMode(IN4, OUTPUT);
  pinMode(ENA, OUTPUT);
  pinMode(ENB, OUTPUT);
  pinMode(LineTeacking_Pin_Right, INPUT);
  pinMode(LineTeacking_Pin_Middle, INPUT);
  pinMode(LineTeacking_Pin_Left, INPUT);
  set_motors(0, 0);
 // set_motors(-200, -200);
}

void loop() {
  getBTData();
  bluetooth_mode();

}
