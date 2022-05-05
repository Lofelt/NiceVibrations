#if (UNITY_ANDROID && !UNITY_EDITOR)

using System;
using UnityEngine;

namespace Lofelt.NiceVibrations
{
    // Android JNI call wrappers that are more efficient than AndroidJavaObject::Call()
    //
    // Calling a method via AndroidJavaObject, e.g. `lofeltHaptics.Call("play")`, is inefficient:
    // - It looks up the method by name for each call
    // - It allocates memory during method lookup and argument conversion
    //
    // JNIHelpers provides alternative Call() methods that are more efficient:
    // - It allows calling by method ID rather by method name, so that the method only needs to
    //   be looked up once, not for every call
    // - It does not allocate memory for converting the arguments to jvalue[]
    //
    // In addition to that, exceptions thrown in Java are handled automatically by logging them.
    //
    // The Call() overload here do not cover all cases that AndroidJavaObject::Call() covers. For
    // example, only methods with one argument are supported, and that only for certain types. In
    // addition, not all overloads are free of allocations. This however is good enough so that the
    // calls triggered by common playback scenarios such as HapticController::Play() and
    // HapticPatterns::PlayPreset() don't allocate.
    internal static class JNIHelpers
    {
        // The array for the JNI arguments is created here, so that it doesn't need to be created
        // for every call. This saves the allocation in each call.
        // The array supports only methods with 0 or 1 argument, but that covers our needs.
        static jvalue[] jniArgs = new jvalue[1];

        // Returns an exception message and stack trace for the given Java exception
        static String javaThrowableToString(IntPtr throwable)
        {
            IntPtr throwableClass = AndroidJNI.FindClass("java/lang/Throwable");
            IntPtr androidUtilLogClass = AndroidJNI.FindClass("android/util/Log");
            try
            {
                IntPtr toStringMethodId = AndroidJNI.GetMethodID(throwableClass, "toString", "()Ljava/lang/String;");
                IntPtr getStackTraceStringMethodId = AndroidJNI.GetStaticMethodID(androidUtilLogClass, "getStackTraceString", "(Ljava/lang/Throwable;)Ljava/lang/String;");
                string exceptionMessage = AndroidJNI.CallStringMethod(throwable, toStringMethodId, new jvalue[] { });
                jniArgs[0].l = throwable;
                string exceptionCallStack = AndroidJNI.CallStaticStringMethod(androidUtilLogClass, getStackTraceStringMethodId, jniArgs);
                return exceptionMessage + "\n" + exceptionCallStack;
            }
            finally
            {
                if (throwable != IntPtr.Zero)
                    AndroidJNI.DeleteLocalRef(throwable);
                if (throwableClass != IntPtr.Zero)
                    AndroidJNI.DeleteLocalRef(throwableClass);
                if (androidUtilLogClass != IntPtr.Zero)
                    AndroidJNI.DeleteLocalRef(androidUtilLogClass);
            }
        }

        public static void Call(AndroidJavaObject obj, IntPtr methodId, jvalue[] jniArgs)
        {
            if (methodId == IntPtr.Zero)
            {
                return;
            }

            try
            {
                AndroidJNI.CallVoidMethod(obj.GetRawObject(), methodId, jniArgs);
                IntPtr throwable = AndroidJNI.ExceptionOccurred();
                if (throwable != IntPtr.Zero)
                {
                    AndroidJNI.ExceptionClear();
                    String exception = javaThrowableToString(throwable);
                    Debug.LogError(exception);
                }
            }
            catch (Exception ex)
            {
                Debug.LogException(ex);
            }
        }

        public static void Call(AndroidJavaObject obj, IntPtr methodId)
        {
            jniArgs[0].l = System.IntPtr.Zero;
            Call(obj, methodId, jniArgs);
        }

        public static void Call(AndroidJavaObject obj, IntPtr methodId, float arg)
        {
            jniArgs[0].f = arg;
            Call(obj, methodId, jniArgs);
        }

        public static void Call(AndroidJavaObject obj, IntPtr methodId, bool arg)
        {
            jniArgs[0].z = arg;
            Call(obj, methodId, jniArgs);
        }

        public static void Call(AndroidJavaObject obj, IntPtr methodId, float[] arg)
        {
            // The allocations in the next two lines could probably be removed to optimize this
            // further.
            object[] args = new object[] { arg };
            jvalue[] jniArgs = AndroidJNIHelper.CreateJNIArgArray(args);
            try
            {
                JNIHelpers.Call(obj, methodId, jniArgs);
            }
            finally
            {
                AndroidJNIHelper.DeleteJNIArgArray(args, jniArgs);
            }
        }

        // The method isn't yet optimized to reduce allocations, but unlike the other overloads of
        // Call(), it supports non-void return types.
        public static ReturnType Call<ReturnType>(AndroidJavaObject obj, string methodName)
        {
            try
            {
                return obj.Call<ReturnType>(methodName);
            }
            catch (Exception ex)
            {
                Debug.LogException(ex);
                return default(ReturnType);
            }
        }

    }
}
#endif