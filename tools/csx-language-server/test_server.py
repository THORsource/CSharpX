import unittest
from server import declarations, diagnostics, word
class TestServer(unittest.TestCase):
    def test_declarations(self):
        self.assertIn('Animal',[x[0] for x in declarations('class Animal { public int Speak() { return 1; } }')])
        self.assertIn('Speak',[x[0] for x in declarations('class Animal { public int Speak() { return 1; } }')])
    def test_diagnostics(self):
        self.assertEqual([],diagnostics('class A {}'))
        self.assertEqual('CSX1002',diagnostics('class A {')[0]['code'])
    def test_word(self):
        self.assertEqual('Dog',word('Dog dog;',{'line':0,'character':1})[0])
if __name__=='__main__':unittest.main()
