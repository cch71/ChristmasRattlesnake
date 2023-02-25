 /*
 * PIR sensor tester
 */

#define LED_PIN           (13)         // choose the pin for the LED
#define MOTION_INPUT_PIN  (8)          // choose the input pin (for PIR sensor)
#define RANGE_TRIGGER_PIN (3)          // Arduino pin tied to trigger pin on the ultrasonic sensor.
#define RANGE_ECHO_PIN    (2)          // Arduino pin tied to echo pin on the ultrasonic sensor.
#define MAX_DISTANCE      (500)        // Maximum distance we want to ping for (in centimeters). Maximum sensor distance is rated at 400-500cm.
#define RLY_SWITCH_CH1    (4)          // Digital Relay Switch Channel 1

#define TIME_BETWEEN_RANGE_READS (40)

enum snake_speed {
  SNAKE_SLEEP = 0,
  SNAKE_FAST = 1,
  SNAKE_MED = 2,
  SNAKE_SLOW = 3
};

#include <NewPing.h>

int pir_state = LOW;             // we start, assuming no motion detected
int motion_input_pin = 0;                    // variable for reading the pin status
unsigned int next_time_to_read_distance = 0;
snake_speed snake_awake = SNAKE_SLEEP;
unsigned int stay_on_delay_start = 0;
unsigned int stay_on_delay_duration = 0;
bool is_in_off_delay = false;

NewPing sonar(RANGE_TRIGGER_PIN, RANGE_ECHO_PIN, MAX_DISTANCE);

void setup() {
  pinMode(LED_PIN, OUTPUT);      // declare LED as output
  pinMode(MOTION_INPUT_PIN, INPUT);     // declare sensor as input
  pinMode(RLY_SWITCH_CH1, OUTPUT);      // declare Relay Switch as output

  Serial.begin(115200);
  digitalWrite(RLY_SWITCH_CH1, HIGH); // turn LED OFF
  //digitalWrite(RLY_SWITCH_CH1, LOW);

  //Serial.begin(9600);
}

void set_snake(const snake_speed speed) {
  if (speed!=snake_awake) {
    snake_awake = speed;
    Serial.print("SNAKE ");
    Serial.println(speed);
  }
}

void handle_snake() {
  unsigned int current_ms = millis();
  if (stay_on_delay_start) {
    if( (current_ms - stay_on_delay_start) >= stay_on_delay_duration) {
      Serial.print("Snake Off :");
      Serial.print(current_ms);
      Serial.print(" ");
      Serial.print(stay_on_delay_start);
      Serial.print(" ");
      Serial.println(stay_on_delay_duration);
      stay_on_delay_start = 0;
    } else if ( (current_ms - stay_on_delay_start) >= (stay_on_delay_duration/2)) {
      if (!is_in_off_delay) {
        Serial.println("Snake Off Delay");
        is_in_off_delay = true;
        digitalWrite(RLY_SWITCH_CH1, HIGH); // turn LED OFF
      }
    }
    return;
  }

  if (SNAKE_SLOW == snake_awake) {
    stay_on_delay_duration = 1000;
  } else if(SNAKE_MED == snake_awake) {
    stay_on_delay_duration = 500;
  } else if (SNAKE_FAST == snake_awake) {
    stay_on_delay_duration = 60;
  } else if (SNAKE_SLEEP == snake_awake) {
    return;
    // Serial.print("Snake Sleeping");
    // stay_on_delay_start = 0;
    // digitalWrite(RLY_SWITCH_CH1, HIGH); // turn LED OFF
  }
  Serial.print("Snake On: \n");
  stay_on_delay_start = current_ms!=0?current_ms:1;
  digitalWrite(RLY_SWITCH_CH1, LOW); // turn LED ON
  is_in_off_delay = false;
}

void loop(){

  //Handle distance reading
  unsigned int current_ms = millis();
  if ( (current_ms > next_time_to_read_distance) && (HIGH==motion_input_pin)) { //This is bad for wraparound
    next_time_to_read_distance = current_ms + TIME_BETWEEN_RANGE_READS;
    // Wait between pings (about 20 pings/sec). 29ms should be the shortest delay between pings.
    int current_distance = sonar.ping_cm();

    //Serial.print("Ping: ");
    //Serial.print(current_distance); // Send ping, get distance in cm and print result (0 = outside set distance range)
    //Serial.println("cm");

    if (170 <= current_distance) {
      set_snake(SNAKE_SLOW);
    } else if ((169 > current_distance) && (30 <= current_distance)) {
      set_snake(SNAKE_MED);
    } else if ((29 > current_distance) && (0 <= current_distance)) {
      set_snake(SNAKE_FAST);
    }
  }

  //Watch for Motion
  motion_input_pin = digitalRead(MOTION_INPUT_PIN);  // read input value
  if (motion_input_pin == HIGH) {            // check if the input is HIGH
    digitalWrite(LED_PIN, HIGH);  // turn LED ON
    //digitalWrite(RLY_SWITCH_CH1, LOW);

    if (pir_state == LOW) {
      // we have just turned on
      Serial.println("Motion detected!");
      // We only want to print on the output change, not state
      pir_state = HIGH;
    }
  } else {
    digitalWrite(LED_PIN, LOW); // turn LED OFF
    //digitalWrite(RLY_SWITCH_CH1, HIGH);
    if (pir_state == HIGH){
      // we have just turned of
      Serial.println("Motion ended!");
      // We only want to print on the output change, not state
      pir_state = LOW;
      set_snake(SNAKE_SLEEP);
    }
  }

  handle_snake();

}
