import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '1m',
};

const BASE_URL = 'http://localhost:8081/birthday';

export default function () {
  const snowflake = `snowflake_${__VU}_${__ITER}`;
  const payload = JSON.stringify({
    snowflake: snowflake,
    name: `User ${__VU}`,
    day: 1,
    month: 1,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
    },
  };

  // 1. Create birthday
  const createRes = http.post(BASE_URL, payload, params);
  check(createRes, {
    'create birthday status is 201': (r) => r.status === 201,
  });

  // 2. Get birthday
  const getRes = http.get(`${BASE_URL}/${snowflake}`, params);
  check(getRes, {
    'get birthday status is 200': (r) => r.status === 200,
    'get birthday has correct snowflake': (r) => r.json().snowflake === snowflake,
  });

  sleep(1);
}
