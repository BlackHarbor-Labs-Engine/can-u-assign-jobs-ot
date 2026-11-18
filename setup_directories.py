import os

dirs = [
    'src/services/user',
    'src/services/auth',
    'src/services/language',
    'src/models'
]

for d in dirs:
    os.makedirs(d, exist_ok=True)

print('Directory structure created successfully.')
