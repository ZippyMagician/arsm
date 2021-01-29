import unittest

fromBytes = lambda n: int.from_bytes(n, signed = True, byteorder = __import__('sys').byteorder)
popN = lambda n, count: [n.pop() for i in range(count)][::-1]

stk = [10, 3, 1, 56, 1, 67]

class TestInline(unittest.TestCase):
    # Ensures fromBytes and popN are working properly
    def testBytesPop(self):         
        self.assertEqual(fromBytes(popN(stk, 4)), 1124153345)
        self.assertEqual(len(stk), 2)
  
if __name__ == '__main__': 
    unittest.main() 