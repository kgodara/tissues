import re
import json

def minutes_to_decimal(mins):
    mins_num = int(mins)
    return mins_num / 60

# Using readlines()
file1 = open('./timezone_html_options.txt', 'r')
Lines = file1.readlines()
 
count = 0
timezones = []
# Strips the newline character
for line in Lines:
    count += 1
    # replace any 'En Dash' chars with hyphens
    cleaned_line = re.sub(u"\u2013", "-", line)
    
    location_name_match = re.search('value=".*" ', cleaned_line)
    gmt_time_match = re.search(r'GMT[-+][\d:]{4,5}', cleaned_line)
    time_name_match = re.search(' - .*<', cleaned_line)

    if location_name_match is None or gmt_time_match is None or time_name_match is None:
        print("Parse Error, Exiting")
        print("cleaned_line: ")
        print(cleaned_line)
        print()
        print(location_name_match)
        print(gmt_time_match)
        print(time_name_match)
        exit()

    location_name = location_name_match.group(0).replace('value=', '').replace('"', '').strip()
    gmt_time = gmt_time_match.group(0)
    time_name = time_name_match.group(0).replace(' - ', '').replace('<', '')

    t_hour_offset = re.search(r'[-+]\d{1,2}', gmt_time)
    t_min_offset = re.search(r'\d{1,2}$', gmt_time)

    t_hour_offset = t_hour_offset.group(0)
    t_min_offset = t_min_offset.group(0)

    decimal_offset = round(float(t_hour_offset) + minutes_to_decimal(t_min_offset), 2)


    timezones.append({"location": location_name, "gmt_t": gmt_time,
                        "t_name": time_name, "decimal_offset": decimal_offset
                    })

print(timezones[0])


with open('./timezones.json', 'w') as fout:
    json.dump(timezones, fout, indent=2)