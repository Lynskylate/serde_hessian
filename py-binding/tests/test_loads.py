import math

import hessian_codec
import io

import unittest



class HessianLoadsTest(unittest.TestCase):
    def test_load_bytes(self):
        self.assertEqual(hessian_codec.loads(b"\x20"),  b"")
        self.assertEqual(hessian_codec.loads(b"\x23\x01\x02\x03"), b"\x01\x02\x03")

    def test_boolean(self):
        self.assertEqual(hessian_codec.loads(b"T"), True)
        self.assertEqual(hessian_codec.loads(b"F"), False)

    def test_null(self):
        self.assertEqual(hessian_codec.loads(b"N"), None)

    def test_int(self):
        self.assertEqual(hessian_codec.loads(b"I\x00\x00\x00\x00"), 0)
        self.assertEqual(hessian_codec.loads(b"\x90"), 0)

    def test_string(self):
        self.assertEqual(hessian_codec.loads(b"S\x00\x00"), "")
        self.assertEqual(hessian_codec.loads(b"S\x00\x01\x00"), "\x00")

    def test_date(self):
        # deserialize bytes to python datetime
        import datetime
        self.assertEqual(hessian_codec.loads(b"\x4a\x00\x00\x00\xd0\x4b\x92\x84\xb8"), datetime.datetime(1998, 5, 8, 9, 51, 31, tzinfo=datetime.timezone.utc))


    def test_list(self):
        self.assertEqual(hessian_codec.loads(b"V\x04[int\x92\x90\x91"), [0, 1])


    def test_dict(self):
        self.assertEqual(hessian_codec.loads(b"H\x91\x03fee\xa0\x03fie\xc9\x00\x03foeZ"), {1: "fee", 16: "fie", 256: "foe"})

    def test_struct(self):
        self.assertEqual(hessian_codec.loads(b"C\x0bexample.Car\x92\x05Color\x05ModelO\x90\x03red\x08corvette"), {"Color": "red", "Model": "corvette"})


if __name__ == '__main__':
    unittest.main()