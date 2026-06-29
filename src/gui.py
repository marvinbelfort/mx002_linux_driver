import tkinter
import json 
# Value for pen pressure
pen_pressure= int(0)
# Value for pen's proximity. 
proximity_threshold= int(0)

# Generate json for output.


def generate_json(pen_pressure,proximity_threshold):

    pen_pressure = int(pen_pressure)
    proximity_threshold = int(pen_pressure)

    # Verify limits
    # pen_pressure E [1,10]
    # threshold_proximity E [0,1700]

    # Out of bounds
    if(proximity_threshold < 0 or proximity_threshold > 1700):
        print('gui.py: Invalid value for proximity.')
        exit()

    elif(pen_pressure < 0 or pen_pressure > 1700):
        print('gui.py: Invalid value for pen_pressure.')
        exit()

    # Generates json output
    json_out = {
    "pen_pressure":str(pen_pressure),
    "proximity_threshold":str(proximity_threshold)
    }

    # Generates json file
    print('gui.py: dumping json...')
    with open("config.json", "w", encoding="utf-8") as json_file:
        json.dump(json_out, json_file, indent=4, ensure_ascii=False)
        



generate_json(1,2)
    

