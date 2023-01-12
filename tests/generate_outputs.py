import subprocess as sp
import glob

EXEC = "../target/release/myton"

def generate_outputs(file_name):
    print('Generating outputs for', file_name)
    out = None
    try:
        out = sp.check_output([EXEC, file_name], universal_newlines=True)
    except sp.CalledProcessError as e:
        out = e.output
    with open(file_name.removesuffix(".my") + '.out', 'w') as f:
        f.write(str(out))

def main():
    sp.call(['cargo', 'build', '--release'], stdout=sp.DEVNULL, stderr=sp.DEVNULL)
    for file_name in glob.glob('**/*.my'):
        generate_outputs(file_name)

if __name__ == '__main__':
    main()
    
