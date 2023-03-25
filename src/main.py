import json
import uos

def main():
    print('#')
    
    while True:
        try:
            i = json.loads(input())
            if i['command'] == 'cat':
                print(json.dumps({
                    'success': cat(i['path'])
                }))

            if i['command'] == 'ls':
                print(json.dumps({
                    'success': ls(i['path'])
                }))

            if i['command'] == 'exists':
                print(json.dumps({
                    'success': exists(i['path'])
                }))

            if i['command'] == 'exit':
                break

        except Exception as error:
            print(json.dumps({
                'error': str(error)
            }))


def exists(path):
    try:
        uos.stat(path)
        return True
    except:
        return False

def ls(path):
    result = []
    for f in uos.ilistdir(path):
        result.append(f[0])
    return result


def cat(path):
    res = ""
    with open(path) as f:
        while True:
            b = f.read(100)
            if not b:
                break
            res += b
    return res
             

main()