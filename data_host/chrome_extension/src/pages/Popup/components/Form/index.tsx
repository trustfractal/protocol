import React from 'react';
import { useDispatch } from 'react-redux';
import { Form as FinalForm, Field, FormRenderProps } from 'react-final-form';
import { FormApi } from 'final-form';

import Error from '@components/Error';
import Section from '@components/Section';
import Spacing from '@components/Spacing';

import { addFractalID } from '@redux/actions';

type FormData = Record<string, any>;
type FormFunction = FormApi<FormData, Partial<Record<string, any>>>;

const onSubmit = async (
  dispatch: Function,
  { fractalID }: FormData,
  form: FormFunction
) => {
  await dispatch(addFractalID(fractalID));
  form.reset();
};

const validate = (data: FormData) => {
  const { fractalID } = data;

  switch (true) {
    case Object.keys(data).length === 0:
      return {};
    case !fractalID.startsWith('0x'):
      return {
        fractalID: 'Expected hex-encoded byte sequence beginning with "0x"',
      };
    default:
      return {};
  }
};

const render = ({ handleSubmit }: FormRenderProps) => (
  <form onSubmit={handleSubmit}>
    <Field name="fractalID">
      {({ input, meta }) => (
        <Section>
          <input {...input} type="text" placeholder="Fractal ID" />
          {meta.error && meta.touched && <Error>{meta.error}</Error>}
        </Section>
      )}
    </Field>

    <Spacing size="s" />
    <button type="submit">Set ID</button>
  </form>
);

const Form = () => {
  const dispatch = useDispatch();

  return (
    <FinalForm
      onSubmit={async (data: FormData, form: FormFunction) =>
        await onSubmit(dispatch, data, form)
      }
      validate={validate}
      render={render}
    />
  );
};

export default Form;
