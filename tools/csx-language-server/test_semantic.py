import unittest
from semantic import Workspace

A='file:///demo/Animal.csx'
B='file:///demo/Dog.csx'
C='file:///demo/Program.csx'

class SemanticTests(unittest.TestCase):
    def setUp(self):
        self.w=Workspace()
        self.w.update(A,'class Animal { public virtual int Speak() { return 1; } public int Age; }')
        self.w.update(B,'class Dog : Animal { public override int Speak() { return 2; } }')
        self.w.update(C,'class Program { public void Run() { Dog d = new Dog(); d.Speak(); d.Age; } }')
    def resolve(self,uri,word,from_end=False):
        src=self.w.docs[uri];at=src.rindex(word) if from_end else src.index(word)
        return self.w.resolve(uri,at)
    def test_cross_file_definition(self):
        self.assertEqual(A,self.resolve(C,'Age')[0].uri)
        self.assertEqual(B,self.resolve(C,'Speak')[0].uri)
    def test_inheritance_completion(self):
        src=self.w.docs[C];at=src.index('d.Age')+2
        self.assertIn('Speak',[s.name for s in self.w.completions(C,at)])
        self.assertIn('Age',[s.name for s in self.w.completions(C,at)])
    def test_override_and_cycles(self):
        self.assertEqual([],self.w.diagnostics(B))
        self.w.update(B,'class Dog : Animal { public override int Missing() { return 2; } }')
        self.assertIn('CSX2205',[d['code'] for d in self.w.diagnostics(B)])
        self.w.update(A,'class Animal : Dog {}')
        self.assertIn('CSX2203',[d['code'] for d in self.w.diagnostics(A)])
    def test_occurrences_avoid_same_spelling(self):
        sy=self.resolve(C,'Speak')[0]
        refs=self.w.occurrences(sy)
        self.assertEqual({B,C},{u for u,_,_ in refs})
    def test_local_shadowing(self):
        self.w.update(C,'class Program { int x; void Run() { int x = 3; x = 4; } }')
        s=self.resolve(C,'x = 4')[0]
        self.assertEqual('local',s.kind)

if __name__=='__main__':unittest.main()
