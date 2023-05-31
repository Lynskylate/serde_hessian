import unittest
import hessian_py

class HessianRoundTripTest(unittest.TestCase):
    def roundtrip(self, val):
        self.assertEqual(val, hessian_py.loads(hessian_py.dumps(val)))

    def test_load_bytes(self):
        self.roundtrip(b"")
        self.roundtrip(b"\x01\x02\x03")
        self.roundtrip(b"\x01" * 256)

    def test_boolean(self):
        self.roundtrip(True)
        self.roundtrip(False)

    def test_null(self):
        self.roundtrip(None)

    def test_int(self):
        self.roundtrip(0)
        self.roundtrip(1)
        self.roundtrip(-1)

    def test_string(self):
        self.roundtrip("")
        self.roundtrip("test")

    def test_date(self):
        import datetime
        self.roundtrip(datetime.datetime(1998, 5, 8, 9, 51, 31, tzinfo=datetime.timezone.utc))


    def test_list(self):
        self.roundtrip([0, 1])


    def test_dict(self):
        self.roundtrip({"test": 1, "test2": 2})

if __name__ == '__main__':
    unittest.main()